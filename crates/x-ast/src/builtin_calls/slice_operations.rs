#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceReceiverAccess {
    Shared,
    Mutable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceOperation {
    Len,
    IsEmpty,
    AsPointer,
    AsMutPointer,
    AsConst,
    Iter,
    Remaining,
    Next,
    First,
    Last,
    SplitAt,
    Sub,
    Reduce,
    Sum,
    All,
    Any,
    Contains,
    HasDuplicateBy,
    UniqueBy,
    Fill,
    Swap,
    CopyWithin,
    CopyFrom,
    Get,
    GetMut,
}

impl SliceOperation {
    pub const fn receiver_access(self) -> SliceReceiverAccess {
        match self {
            Self::AsMutPointer
            | Self::Next
            | Self::Fill
            | Self::Swap
            | Self::CopyWithin
            | Self::CopyFrom
            | Self::GetMut => SliceReceiverAccess::Mutable,
            Self::Len
            | Self::IsEmpty
            | Self::AsPointer
            | Self::AsConst
            | Self::Iter
            | Self::Remaining
            | Self::First
            | Self::Last
            | Self::SplitAt
            | Self::Sub
            | Self::Reduce
            | Self::Sum
            | Self::All
            | Self::Any
            | Self::Contains
            | Self::HasDuplicateBy
            | Self::UniqueBy
            | Self::Get => SliceReceiverAccess::Shared,
        }
    }

    pub const fn source_name(self) -> &'static str {
        match self {
            Self::Len => "len",
            Self::IsEmpty => "is_empty",
            Self::AsPointer => "as_ptr",
            Self::AsMutPointer => "as_mut_ptr",
            Self::AsConst => "as_const",
            Self::Iter => "iter",
            Self::Remaining => "remaining",
            Self::Next => "next",
            Self::First => "first",
            Self::Last => "last",
            Self::SplitAt => "split_at",
            Self::Sub => "sub",
            Self::Reduce => "reduce",
            Self::Sum => "sum",
            Self::All => "all",
            Self::Any => "any",
            Self::Contains => "contains",
            Self::HasDuplicateBy => "has_duplicate_by",
            Self::UniqueBy => "unique_by",
            Self::Fill => "fill",
            Self::Swap => "swap",
            Self::CopyWithin => "copy_within",
            Self::CopyFrom => "copy_from",
            Self::Get => "get",
            Self::GetMut => "get_mut",
        }
    }
}
