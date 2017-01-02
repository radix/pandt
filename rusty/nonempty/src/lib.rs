#![feature(proc_macro)]
// use std::fmt;

#[macro_use]
extern crate serde_derive;
extern crate serde;
#[cfg(test)]
extern crate serde_json;

use serde::{Serialize, Serializer, Deserialize, Deserializer};
#[cfg(test)]
use serde_json;
#[cfg(test)]
use serde_json::error as SJE;


// TODO: implement iter() etc, and ... ALL the rest of the Vec methods... :(
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct NonEmptyWithCursor<T> {
    most: Vec<T>,
    head: T,
    cursor: usize,
}

impl<T> NonEmptyWithCursor<T> {
    pub fn new(head: T) -> NonEmptyWithCursor<T> {
        NonEmptyWithCursor {
            most: vec![],
            head: head,
            cursor: 0,
        }
    }
    pub fn get_current(&self) -> &T {
        if self.cursor == 0 {
            &self.head
        } else {
            &self.most[self.cursor - 1]
        }
    }

    pub fn get_current_mut(&mut self) -> &mut T {
        let i = self.cursor;
        self.get_mut(i).unwrap()
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx == 0 {
            Some(&self.head)
        } else {
            self.most.get(idx - 1)
        }
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx == 0 {
            Some(&mut self.head)
        } else {
            self.most.get_mut(idx - 1)
        }
    }

    pub fn push(&mut self, t: T) {
        self.most.push(t)
    }

    pub fn set_cursor(&mut self, cursor: usize) -> Option<()> {
        if self.most.len() < cursor {
            None
        } else {
            self.cursor = cursor;
            Some(())
        }
    }

    pub fn next_circle(&mut self) {
        let newcursor = self.cursor + 1;
        self.cursor = if newcursor > self.most.len() {
            0
        } else {
            newcursor
        }
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }

    pub fn len(&self) -> usize {
        1 + self.most.len()
    }
}

impl<T> Serialize for NonEmptyWithCursor<T>
    where T: Serialize
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        // We have to make a helper struct to serialize the value of `data`, because we have to
        // have something to pass to `serialize_map_value`.
        struct Data<'a, T2>(&'a NonEmptyWithCursor<T2>) where T2: 'a;
        impl<'a, T2> Serialize for Data<'a, T2>
            where T2: Serialize
        {
            fn serialize<S2>(&self, serializer: &mut S2) -> Result<(), S2::Error>
                where S2: Serializer
            {
                let mut state = serializer.serialize_seq(Some(self.0.len()))?;
                serializer.serialize_seq_elt(&mut state, &self.0.head)?;
                for elt in self.0.most.iter() {
                    serializer.serialize_seq_elt(&mut state, elt)?;
                }
                serializer.serialize_seq_end(state)
            }
        }
        let mut state = serializer.serialize_map(Some(2))?;
        serializer.serialize_map_key(&mut state, "cursor")?;
        serializer.serialize_map_value(&mut state, self.cursor)?;
        serializer.serialize_map_key(&mut state, "data")?;
        serializer.serialize_map_value(&mut state, Data(&self))?;
        serializer.serialize_map_end(state)
    }
}

#[derive(Deserialize)]
struct SerializedNE<T> {
    cursor: usize,
    data: Vec<T>,
}

// Can't figure out how to use custom error types in serde deserialization

// #[derive(Eq, PartialEq, Debug)]
// pub enum DeserializeNonEmptyError {
//     InvalidCursor(usize, usize),
//     Empty,
// }
//
// impl fmt::Display for DeserializeNonEmptyError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             &DeserializeNonEmptyError::InvalidCursor(cursor, size) => {
//                 write!(f,
//                        "Cursor of {} should not be larger than length {}",
//                        cursor,
//                        size)
//             }
//             &DeserializeNonEmptyError::Empty => write!(f, "data must have at least one element"),
//         }
//     }
// }

impl<T> Deserialize for NonEmptyWithCursor<T>
    where T: Deserialize
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let mut x: SerializedNE<T> = Deserialize::deserialize(deserializer)?;
        if x.data.len() == 0 {
            return Err(serde::de::Error::invalid_length(0));
        }
        let most: Vec<T> = x.data.drain(1..).collect();
        if x.cursor > most.len() {
            return Err(serde::de::Error::invalid_value(&format!("Cursor of {} should not be \
                                                                larger than length {}",
                                                                x.cursor,
                                                                most.len() + 1)));
        }
        let res: NonEmptyWithCursor<T> = NonEmptyWithCursor {
            cursor: x.cursor,
            head: x.data.pop().unwrap(),
            most: most,
        };
        Ok(res)

    }
}


#[test]
fn test_nonempty_set_cursor() {
    let mut ne: NonEmptyWithCursor<i32> = NonEmptyWithCursor::new(1);
    assert_eq!(ne.get_cursor(), 0);
    assert_eq!(ne.get_current(), &1);

    assert_eq!(ne.set_cursor(0), Some(()));
    assert_eq!(ne.get_cursor(), 0);
    assert_eq!(ne.get_current(), &1);

    assert_eq!(ne.set_cursor(1), None);
    assert_eq!(ne.get_cursor(), 0);
    assert_eq!(ne.get_current(), &1);

    ne.push(5);
    assert_eq!(ne.get_cursor(), 0);
    assert_eq!(ne.get_current(), &1);

    assert_eq!(ne.set_cursor(1), Some(()));
    assert_eq!(ne.get_cursor(), 1);
    assert_eq!(ne.get_current(), &5);
}


#[test]
fn test_nonempty_serialize_deserialize() {
    let mut ne: NonEmptyWithCursor<i32> = NonEmptyWithCursor::new(5);
    ne.push(50);
    ne.push(55);
    assert_eq!(serde_json::to_string(&ne).unwrap(),
               "{\"cursor\":0,\"data\":[5,50,55]}");
    match serde_json::from_str("{\"cursor\":0,\"data\":[5,50,55]}") {
        Ok(ne2) => assert_eq!(ne, ne2),
        Err(e) => panic!("Couldn't parse json: {}", e),
    }
}

#[test]
fn test_deserialize_invalid_cursor() {
    let res: Result<NonEmptyWithCursor<i32>, _> = serde_json::from_str("{\"cursor\":2,\"data\":\
                                                                        [5]}");
    let exmsg = "Cursor of 2 should not be larger than length 1";
    match res {
        Ok(x) => panic!("Should not have parsed to {:?}", x),
        // TODO: position here is 0, 0 because our parser is dumb.
        Err(SJE::Error::Syntax(SJE::ErrorCode::InvalidValue(ref msg), 0, 0)) if msg == exmsg => {}
        Err(e) => panic!("Should not have got any other error: {:?}", e),
    }
}

#[test]
fn test_deserialize_invalid_empty() {
    let res: Result<NonEmptyWithCursor<i32>, _> = serde_json::from_str("{\"cursor\":0,\"data\":\
                                                                        []}");
    match res {
        Ok(x) => panic!("Should not have parsed to {:?}", x),
        // TODO: position here is 0, 0 because our parser is dumb.
        Err(SJE::Error::Syntax(SJE::ErrorCode::InvalidLength(0), 0, 0)) => {}
        Err(e) => panic!("Should not have got any other error: {}", e),
    }
}