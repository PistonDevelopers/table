#![deny(missing_docs)]
#![feature(core, std_misc, hash)]

//! A table object type for dynamical data

use std::collections::HashMap;
use std::sync::Arc;
use std::ops::{ Deref, DerefMut, Index, IndexMut };
use std::hash::{ Hash, Hasher, Writer };
use std::borrow::BorrowFrom;

/// Represents a dynamical typed value
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    /// An empty value.
    Null,
    /// A boolean value.
    Bool(bool),
    /// A pointer sized integer.
    Usize(usize),
    /// A 64 bit unsigned integer.
    U64(u64),
    /// A 64 bit signed integer.
    I64(i64),
    /// A 64 bit floating number.
    F64(F64),
    /// A string.
    String(Arc<String>),
    /// A table.
    Table(Arc<Table>),
}

impl Value {
    /// Creates a new f64 value.
    pub fn f64(val: f64) -> Value {
        Value::F64(F64(val))
    }

    /// Creates a new string value.
    pub fn str(val: &str) -> Value {
        use std::borrow::ToOwned;

        Value::String(Arc::new(val.to_owned()))
    }

    /// Creates a new table value.
    pub fn table(val: Table) -> Value {
        Value::Table(Arc::new(val))
    }
}

impl<S> Hash<S> for Value
    where
        S: Hasher + Writer
{
    fn hash(&self, state: &mut S) {
        match self {
            &Value::String(ref text) => text.hash(state),
            &Value::Null => 0us.hash(state),
            &Value::Bool(val) => val.hash(state),
            &Value::Usize(val) => val.hash(state),
            &Value::U64(val) => val.hash(state),
            &Value::I64(val) => val.hash(state),
            &Value::F64(val) => val.hash(state),
            &Value::Table(ref val) => val.hash(state),
        }
    }
}

impl BorrowFrom<Value> for str {
    fn borrow_from(owned: &Value) -> &str {
        match owned {
            &Value::String(ref text) => {
                &text[]
            }
            _ => ""
        }
    }
}

/// Wrapper for f64
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct F64(pub f64);

impl Eq for F64 {}

impl<S> Hash<S> for F64
    where
        S: Hasher + Writer
{
    fn hash(&self, state: &mut S) {
        let val = self.0 as u64;
        val.hash(state)
    }
}

impl Deref for F64 {
    type Target = f64;

    fn deref<'a>(&'a self) -> &'a f64 {
        &self.0
    }
}

impl DerefMut for F64 {
    fn deref_mut<'a>(&'a mut self) -> &'a mut f64 {
        &mut self.0
    }
}

/// The table object
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Table(pub HashMap<Value, Value>);

impl<S> Hash<S> for Table
    where
        S: Hasher + Writer
{
    fn hash(&self, state: &mut S) {
        for (key, val) in self.0.iter() {
            key.hash(state);
            val.hash(state);
        }
    }
}

impl Deref for Table {
    type Target = HashMap<Value, Value>;

    fn deref<'a>(&'a self) -> &'a HashMap<Value, Value> {
        &self.0
    }
}

impl DerefMut for Table {
    fn deref_mut<'a>(&'a mut self) -> &'a mut HashMap<Value, Value> {
        &mut self.0
    }
}

impl Index<Value> for Table {
    type Output = Value;

    fn index<'a>(&'a self, index: &Value) -> &'a Value {
        self.0.get(index).unwrap()
    }
}

impl Index<usize> for Table {
    type Output = Value;

    fn index<'a>(&'a self, index: &usize) -> &'a Value {
        self.0.get(&Value::Usize(*index)).unwrap()
    }
}

impl<'b> Index<&'b str> for Table {
    type Output = Value;

    fn index<'a>(&'a self, index: &&'b str) -> &'a Value {
        self.0.get(*index).unwrap()
    }
}

impl IndexMut<Value> for Table {
    type Output = Value;

    fn index_mut<'a>(&'a mut self, index: &Value) -> &'a mut Value {
        use std::collections::hash_map::Entry;

        match self.0.entry(index.clone()) {
            Entry::Occupied(_) => {},
            Entry::Vacant(x) => {x.insert(Value::Null);}
        };
        self.0.get_mut(index).unwrap()
    }
}

impl IndexMut<usize> for Table {
    type Output = Value;

    fn index_mut<'a>(&'a mut self, index: &usize) -> &'a mut Value {
        use std::collections::hash_map::Entry;

        match self.0.entry(Value::Usize(*index)) {
            Entry::Occupied(_) => {},
            Entry::Vacant(x) => {x.insert(Value::Null);}
        };
        self.0.get_mut(&Value::Usize(*index)).unwrap()
    }
}

impl<'b> IndexMut<&'b str> for Table {
    type Output = Value;

    fn index_mut<'a>(&'a mut self, index: &&'b str) -> &'a mut Value {
        use std::borrow::ToOwned;

        if !self.0.contains_key(*index) {
            self.insert(Value::String(Arc::new((*index).to_owned())), 
                Value::Null);
        }
        self.0.get_mut(*index).unwrap()
    }
}

impl Table {
    /// Creates new table.
    pub fn new() -> Table {
        Table(HashMap::new())
    }

    /// Creates new table with capacity.
    pub fn with_capacity(capacity: usize) -> Table {
        Table(HashMap::with_capacity(capacity))
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;
    use self::test::Bencher;
    use std::sync::Arc;

    #[test]
    fn test_size() {
        use std::mem::size_of;

        // Make sure `Value` is not exceeding 128 bits on 64 bit systems.
        // This is for performance reasons.
        if size_of::<usize>() == 8 {
            assert_eq!(size_of::<Value>(), 16);
        }
    }

    #[test]
    fn test_sync() {
        fn foo<T: Sync>() {}

        // Make sure `Value` can be shared among tasks.
        foo::<Value>();
        foo::<Table>();
    }

    #[test]
    fn test_vec3() {
        let mut vec3 = Table::with_capacity(3);
        vec3[0us] = Value::f64(1.0);
        vec3[1us] = Value::f64(2.0);
        vec3[2us] = Value::f64(3.0);
        assert_eq!(vec3[0us], Value::f64(1.0));
        assert_eq!(vec3[1us], Value::f64(2.0));
        assert_eq!(vec3[2us], Value::f64(3.0));
        vec3.clear();
        assert_eq!(vec3.len(), 0);
    }

    #[test]
    fn test_table_as_key() {
        use std::borrow::ToOwned;

        let a = Value::Table(Arc::new(Table::new()));
        let mut b = Table::new();
        b[a] = Value::String(Arc::new("hello".to_owned()));
    }

    #[test]
    fn test_str_as_key() {
        use std::borrow::ToOwned;

        let mut a = Table::new();
        a.insert(Value::String(Arc::new("hello".to_owned())),
            Value::String(Arc::new("world".to_owned())));
        assert!(
            if let Some(&Value::String(_)) = a.get("hello") { true }
            else { false }
        );
        assert_eq!(a["hello"], Value::String(Arc::new("world".to_owned())));
        a["hello2"] = Value::String(Arc::new("world".to_owned()));
        assert_eq!(a["hello"], a["hello2"]);
    }

    #[bench]
    fn bench_create_empty(bencher: &mut Bencher) {
        bencher.iter(|| {
            let _ = Table::new();
        });
    }
    
    #[bench]
    fn bench_create_empty_arc(bencher: &mut Bencher) {
        use std::sync::Arc;

        bencher.iter(|| {
            let _ = Arc::new(Table::new());
        });
    }
    
    #[bench]
    fn bench_overwrite(bencher: &mut Bencher) {
        let mut a = Table::new();
        bencher.iter(|| {
            a[0us] = Value::f64(1.0);
        });
    }
    
    #[bench]
    fn bench_overwrite_arc(bencher: &mut Bencher) {
        use std::sync::Arc;

        let mut a = Arc::new(Table::new());
        bencher.iter(|| {
            a.make_unique()[0us] = Value::f64(1.0);
        });
    }
}

