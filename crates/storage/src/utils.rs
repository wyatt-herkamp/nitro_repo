macro_rules! new_type_arc_type {
    (
        $ty:ident($from:ty)
    ) => {
        impl From<$from> for $ty {
            fn from(value: $from) -> Self {
                $ty(Arc::new(value))
            }
        }
        impl From<Arc<$from>> for $ty {
            fn from(value: Arc<$from>) -> Self {
                $ty(value)
            }
        }
        impl Deref for $ty {
            type Target = $from;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

pub(crate) use new_type_arc_type;
