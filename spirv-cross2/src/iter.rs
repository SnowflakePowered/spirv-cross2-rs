macro_rules! impl_iterator {
    ($ty:ty: $item:ty as $func:ident |$self:ident, $val:ident: $valtype:ty| $block:block for <$lt:lifetime> [$($slice:tt)+]) => {
        impl<$lt>  ExactSizeIterator for $ty {
            fn len(&self) -> usize {
                self.$($slice)+.len()
            }
        }

        impl<$lt> Iterator for $ty {
            type Item = $item;

            fn next(&mut self) -> Option<Self::Item> {
                self.$($slice)+.next().$func(|val| {
                     (| $self: &mut Self, $val: $valtype| $block )(self, val)
                })
            }
        }

        impl<$lt> DoubleEndedIterator for $ty {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.$($slice)+.next_back().$func(|val| {
                     (| $self: &mut Self, $val: $valtype| $block )(self, val)
                })
            }
        }
    };

    ($ty:ty: $item:ty as $func:ident |$self:ident, $val:ident: $valtype:ty| $block:block for [$($slice:tt)+]) => {
        impl ExactSizeIterator for $ty {
            fn len(&self) -> usize {
                self.$($slice)+.len()
            }
        }

        impl Iterator for $ty {
            type Item = $item;

            fn next(&mut self) -> Option<Self::Item> {
                self.$($slice)+.next().$func(|val| {
                     (| $self: &mut Self, $val: $valtype| $block)(self, val)
                })
            }
        }

        impl DoubleEndedIterator for $ty {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.$($slice)+.next_back().$func(|val| {
                     (| $self: &mut Self, $val: $valtype| $block)(self, val)
                })
            }
        }
    };
}

pub(crate) use impl_iterator;
