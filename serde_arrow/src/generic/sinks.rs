use std::collections::{HashMap, HashSet};

use crate::{
    error,
    base::{Event, EventSink},
    fail, Result,
};

pub trait ArrayBuilder<A>: EventSink {
    fn box_into_array(self: Box<Self>) -> Result<A>;
    fn into_array(self) -> Result<A>
    where
        Self: Sized;
}

pub struct DynamicArrayBuilder<A> {
    builder: Box<dyn ArrayBuilder<A>>,
}

impl<A> DynamicArrayBuilder<A> {
    pub fn new<B: ArrayBuilder<A> + 'static>(builder: B) -> Self {
        Self {
            builder: Box::new(builder),
        }
    }
}

impl<A> EventSink for DynamicArrayBuilder<A> {
    fn accept(&mut self, event: Event<'_>) -> Result<()> {
        self.builder.accept(event)
    }
}

impl<A> ArrayBuilder<A> for DynamicArrayBuilder<A> {
    fn box_into_array(self: Box<Self>) -> Result<A> {
        self.builder.box_into_array()
    }

    fn into_array(self) -> Result<A> {
        self.builder.box_into_array()
    }
}

impl<A> From<Box<dyn ArrayBuilder<A>>> for DynamicArrayBuilder<A> {
    fn from(builder: Box<dyn ArrayBuilder<A>>) -> Self {
        Self { builder }
    }
}

pub struct RecordsBuilder<A> {
    builders: Vec<DynamicArrayBuilder<A>>,
    field_index: HashMap<String, usize>,
    next: State,
    seen: HashSet<usize>,
}

impl<A> RecordsBuilder<A> {
    pub fn new(columns: Vec<String>, builders: Vec<DynamicArrayBuilder<A>>) -> Result<Self> {
        if columns.len() != builders.len() {
            fail!("Number of columns must be equal to the number of builders");
        }

        let mut field_index = HashMap::new();
        for (i, col) in columns.iter().enumerate() {
            if field_index.contains_key(col) {
                fail!("Duplicate field {}", col);
            }
            field_index.insert(col.to_owned(), i);
        }

        Ok(Self {
            builders,
            field_index,
            next: State::StartSequence,
            seen: HashSet::new(),
        })
    }
}

impl<A> RecordsBuilder<A> {
    pub fn into_records(self) -> Result<Vec<A>> {
        if !matches!(self.next, State::Done) {
            fail!("Invalid state");
        }
        let arrays: Result<Vec<A>> = self
            .builders
            .into_iter()
            .map(|builder| builder.into_array())
            .collect();
        let arrays = arrays?;
        Ok(arrays)
    }
}

#[derive(Debug, Clone, Copy)]
enum State {
    StartSequence,
    StartMap,
    Key,
    Value(usize, usize),
    Done,
}

impl<A> EventSink for RecordsBuilder<A> {
    fn accept(&mut self, event: Event<'_>) -> Result<()> {
        use State::*;

        self.next = match (self.next, event.to_self()) {
            (StartSequence, Event::StartSequence) => StartMap,
            (StartMap, Event::EndSequence) => Done,
            (StartMap, Event::StartMap) => {
                self.seen.clear();
                Key
            }
            (Key, Event::Key(k)) => {
                let &idx = self
                    .field_index
                    .get(k)
                    .ok_or_else(|| error!("Unknown field {k}"))?;
                if self.seen.contains(&idx) {
                    fail!("Duplicate field {k}");
                }
                self.seen.insert(idx);

                Value(idx, 0)
            }
            (Key, Event::EndMap) => StartMap,
            // Ignore some events
            (Value(idx, depth), Event::Some) => Value(idx, depth),
            (Value(idx, depth), ev) => {
                let next = match ev {
                    Event::StartSequence | Event::StartMap => Value(idx, depth + 1),
                    Event::EndSequence | Event::EndMap if depth > 1 => Value(idx, depth - 1),
                    Event::EndSequence | Event::EndMap if depth == 0 => fail!("Invalid state"),
                    // the closing event for the nested type
                    Event::EndSequence | Event::EndMap => Key,
                    _ if depth == 0 => Key,
                    _ => Value(idx, depth),
                };

                self.builders[idx].accept(ev)?;
                next
            }
            (state, ev) => fail!("Invalid event {ev} in state {state:?}"),
        };
        Ok(())
    }
}

pub struct StructArrayBuilder<B> {
    pub(crate) columns: Vec<String>,
    pub(crate) nullable: Vec<bool>,
    pub(crate) builders: Vec<B>,
    pub(crate) state: StructArrayBuilderState,
    pub(crate) seen: Vec<bool>,
}

impl<B> StructArrayBuilder<B> {
    pub fn new(columns: Vec<String>, nullable: Vec<bool>, builders: Vec<B>) -> Self {
        let num_columns = columns.len();
        Self {
            columns,
            builders,
            nullable,
            state: StructArrayBuilderState::Start,
            seen: vec![false; num_columns],
        }
    }
}

impl<B: EventSink> EventSink for StructArrayBuilder<B> {
    fn accept(&mut self, event: Event<'_>) -> Result<()> {
        use StructArrayBuilderState::*;

        match self.state {
            Start => match event {
                Event::StartMap => {
                    self.state = Field;
                    self.seen = vec![false; self.columns.len()];
                }
                _ => fail!("Expected start map"),
            },
            Field => {
                let key = match event {
                    Event::Key(key) => key,
                    Event::OwnedKey(ref key) => key,
                    Event::EndMap => {
                        if !self.seen.iter().all(|&seen| seen) {
                            // TODO: improve error message
                            fail!("Missing fields");
                        }
                        self.state = Start;
                        return Ok(());
                    }
                    event => fail!("Unexpected event while waiting for field: {event}"),
                };
                let idx = self
                    .columns
                    .iter()
                    .position(|col| col == key)
                    .ok_or_else(|| error!("unknown field {key}"))?;
                if self.seen[idx] {
                    fail!("Duplicate field {}", self.columns[idx]);
                }
                self.seen[idx] = true;
                self.state = Value(idx, 0);
            }
            Value(active, depth) => {
                self.state = match &event {
                    Event::StartMap | Event::StartSequence => Value(active, depth + 1),
                    Event::EndMap | Event::EndSequence => match depth {
                        // the last closing event for the current value
                        1 => Field,
                        // TODO: check is this event possible?
                        0 => fail!("Unbalanced opening / close events"),
                        _ => Value(active, depth - 1),
                    },
                    _ if depth == 0 => Field,
                    _ => self.state,
                };
                self.builders[active].accept(event)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StructArrayBuilderState {
    Start,
    Field,
    Value(usize, usize),
}
