use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::rc::Rc;
use std::sync::Arc;

use super::JsonSchema;

/// Trait for types that can represent themselves as a JSON JsonSchema.
pub trait IntoSchema {
    fn schema() -> JsonSchema;
}

// =========================================================================
// 1. Примитивные типы
// =========================================================================

macro_rules! impl_schema_primitive {
    ($ty:ty, $constructor:ident) => {
        impl IntoSchema for $ty {
            fn schema() -> JsonSchema {
                JsonSchema::$constructor("")
            }
        }
    };
}

impl_schema_primitive!(String, string);
impl_schema_primitive!(&str, string);
impl_schema_primitive!(bool, boolean);
impl_schema_primitive!(i8, integer);
impl_schema_primitive!(i16, integer);
impl_schema_primitive!(i32, integer);
impl_schema_primitive!(i64, integer);
impl_schema_primitive!(i128, integer);
impl_schema_primitive!(isize, integer);
impl_schema_primitive!(u8, integer);
impl_schema_primitive!(u16, integer);
impl_schema_primitive!(u32, integer);
impl_schema_primitive!(u64, integer);
impl_schema_primitive!(u128, integer);
impl_schema_primitive!(usize, integer);
impl_schema_primitive!(f32, number);
impl_schema_primitive!(f64, number);
impl_schema_primitive!((), null);

// =========================================================================
// 2. Последовательности, списки и множества (JSON Array)
// =========================================================================

// Vec<T>
impl<T: IntoSchema> IntoSchema for Vec<T> {
    fn schema() -> JsonSchema {
        JsonSchema::array("").items(T::schema())
    }
}

// VecDeque<T>, LinkedList<T>, BinaryHeap<T>
macro_rules! impl_schema_seq {
    ($($ty:ident),*) => {
        $(
            impl<T: IntoSchema> IntoSchema for $ty<T> {
                fn schema() -> JsonSchema {
                    JsonSchema::array("").items(T::schema())
                }
            }
        )*
    };
}

impl_schema_seq!(VecDeque, LinkedList, BinaryHeap);

// HashSet<T, S> и BTreeSet<T>
impl<T: IntoSchema, S> IntoSchema for HashSet<T, S> {
    fn schema() -> JsonSchema {
        JsonSchema::array("").items(T::schema())
    }
}

impl<T: IntoSchema> IntoSchema for BTreeSet<T> {
    fn schema() -> JsonSchema {
        JsonSchema::array("").items(T::schema())
    }
}

// Массивы фиксированной длины: [T; N]
impl<T: IntoSchema, const N: usize> IntoSchema for [T; N] {
    fn schema() -> JsonSchema {
        JsonSchema::array("").items(T::schema())
    }
}

// Срезы: &[T]
impl<T: IntoSchema> IntoSchema for &[T] {
    fn schema() -> JsonSchema {
        JsonSchema::array("").items(T::schema())
    }
}

// =========================================================================
// 3. Ассоциативные массивы / Карты (JSON Object)
// =========================================================================

// HashMap<K, V, S>
impl<K, V: IntoSchema, S> IntoSchema for HashMap<K, V, S> {
    fn schema() -> JsonSchema {
        let mut schema = JsonSchema::object("");
        schema.additional_properties = Some(true);
        schema
    }
}

// BTreeMap<K, V>
impl<K, V: IntoSchema> IntoSchema for BTreeMap<K, V> {
    fn schema() -> JsonSchema {
        let mut schema = JsonSchema::object("");
        schema.additional_properties = Some(true);
        schema
    }
}

// =========================================================================
// 4. Опциональные типы, ссылки и умные указатели (Прозрачная передача схемы)
// =========================================================================

impl<T: IntoSchema> IntoSchema for Option<T> {
    fn schema() -> JsonSchema {
        let mut s = T::schema();
        s.optional = Some(true);
        s
    }
}

impl<T: IntoSchema> IntoSchema for Box<T> {
    fn schema() -> JsonSchema {
        T::schema()
    }
}

impl<T: IntoSchema> IntoSchema for Rc<T> {
    fn schema() -> JsonSchema {
        T::schema()
    }
}

impl<T: IntoSchema> IntoSchema for Arc<T> {
    fn schema() -> JsonSchema {
        T::schema()
    }
}

impl<'a, T: ?Sized + ToOwned> IntoSchema for Cow<'a, T>
where
    T::Owned: IntoSchema,
{
    fn schema() -> JsonSchema {
        <T::Owned as IntoSchema>::schema()
    }
}

impl<T: IntoSchema> IntoSchema for Cell<T> {
    fn schema() -> JsonSchema {
        T::schema()
    }
}

impl<T: IntoSchema> IntoSchema for RefCell<T> {
    fn schema() -> JsonSchema {
        T::schema()
    }
}

impl<T: IntoSchema> IntoSchema for &T {
    fn schema() -> JsonSchema {
        T::schema()
    }
}

// =========================================================================
// 5. Кортежи (Tuples)
// =========================================================================

macro_rules! impl_schema_tuple {
    ($($name:ident),+) => {
        impl<$($name: IntoSchema),+> IntoSchema for ($($name,)+) {
            fn schema() -> JsonSchema {
                JsonSchema::array("")
            }
        }
    };
}

impl_schema_tuple!(A);
impl_schema_tuple!(A, B);
impl_schema_tuple!(A, B, C);
impl_schema_tuple!(A, B, C, D);
impl_schema_tuple!(A, B, C, D, E);
