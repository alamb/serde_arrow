use std::collections::HashMap;

use crate::internal::error::Result;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct BitBuffer {
    pub(crate) buffer: Vec<u8>,
    pub(crate) len: usize,
    pub(crate) capacity: usize,
}

impl BitBuffer {
    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, value: bool) -> Result<()> {
        while self.len >= self.capacity {
            for _ in 0..64 {
                self.buffer.push(0);
                self.capacity += 8;
            }
        }

        if value {
            self.buffer[self.len / 8] |= 1 << (self.len % 8);
        }
        self.len += 1;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct NullBuffer {
    len: usize,
}

impl NullBuffer {
    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, _: ()) -> Result<()> {
        self.len += 1;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PrimitiveBuffer<T> {
    pub(crate) buffer: Vec<T>,
}

impl<T> std::default::Default for PrimitiveBuffer<T> {
    fn default() -> Self {
        Self { buffer: Vec::new() }
    }
}

impl<T> PrimitiveBuffer<T> {
    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn push(&mut self, val: T) -> Result<()> {
        self.buffer.push(val);
        Ok(())
    }
}

pub trait Offset: std::ops::Add<Self, Output = Self> + Clone + Default {
    fn try_form_usize(val: usize) -> Result<Self>;
}

impl Offset for i32 {
    fn try_form_usize(val: usize) -> Result<Self> {
        Ok(i32::try_from(val)?)
    }
}

impl Offset for i64 {
    fn try_form_usize(val: usize) -> Result<Self> {
        Ok(i64::try_from(val)?)
    }
}

#[derive(Debug, Clone)]
pub struct OffsetBuilder<O> {
    pub(crate) offsets: Vec<O>,
    pub(crate) current_items: O,
}

impl<O: Offset> std::default::Default for OffsetBuilder<O> {
    fn default() -> Self {
        Self {
            offsets: vec![O::default()],
            current_items: O::default(),
        }
    }
}

impl<O: Offset> OffsetBuilder<O> {
    /// The number of items pushed (one less than the number of offsets)
    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.offsets.len() - 1
    }

    pub fn push(&mut self, num_items: usize) -> Result<()> {
        self.current_items = self.current_items.clone() + O::try_form_usize(num_items)?;
        self.offsets.push(self.current_items.clone());

        Ok(())
    }

    pub fn push_current_items(&mut self) {
        self.offsets.push(self.current_items.clone());
    }

    pub fn inc_current_items(&mut self) -> Result<()> {
        self.current_items = self.current_items.clone() + O::try_form_usize(1)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StringBuffer<O> {
    pub(crate) data: Vec<u8>,
    pub(crate) offsets: OffsetBuilder<O>,
}

impl<O: Offset> std::default::Default for StringBuffer<O> {
    fn default() -> Self {
        Self {
            offsets: Default::default(),
            data: Default::default(),
        }
    }
}

impl<O: Offset> StringBuffer<O> {
    pub fn push(&mut self, val: &str) -> Result<()> {
        self.data.extend(val.as_bytes().iter().copied());
        self.offsets.push(val.len())?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StringDictonary<O> {
    pub(crate) index: HashMap<String, usize>,
    pub(crate) values: StringBuffer<O>,
}

impl<O: Offset> std::default::Default for StringDictonary<O> {
    fn default() -> Self {
        Self {
            index: Default::default(),
            values: Default::default(),
        }
    }
}

impl<O: Offset> StringDictonary<O> {
    pub fn push(&mut self, val: &str) -> Result<usize> {
        if self.index.contains_key(val) {
            Ok(self.index[val])
        } else {
            let res = self.index.len();
            self.index.insert(val.to_string(), res);
            self.values.push(val)?;
            Ok(res)
        }
    }
}
