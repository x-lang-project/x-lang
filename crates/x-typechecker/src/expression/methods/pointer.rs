use x_ast::Type;
use x_ast::intrinsics::PointerOperation;

pub(super) fn resolve_method(name: &str) -> Option<PointerOperation> {
    if matches!(
        name,
        "offset"
            | "add"
            | "sub"
            | "is_null"
            | "addr"
            | "expose_addr"
            | "as_ref"
            | "as_mut"
            | "read"
            | "read_unaligned"
            | "read_volatile"
            | "write"
            | "write_unaligned"
            | "write_volatile"
    ) {
        PointerOperation::resolve(name)
    } else {
        None
    }
}

pub(super) fn signature(operation: PointerOperation, receiver: &Type) -> Option<(Vec<Type>, Type)> {
    let Type::RawPointer { is_mut, inner } = receiver else {
        return None;
    };

    Some(match operation {
        PointerOperation::Offset | PointerOperation::Add | PointerOperation::Subtract => {
            (vec![Type::ISize], receiver.clone())
        }
        PointerOperation::IsNull => (vec![], Type::Boolean),
        PointerOperation::Address => (vec![], Type::USize),
        PointerOperation::AsReference => (
            vec![],
            Type::Reference {
                lifetime: None,
                inner: inner.clone(),
                is_mut: false,
                is_unique: false,
            },
        ),
        PointerOperation::AsMutableReference if *is_mut => (
            vec![],
            Type::Reference {
                lifetime: None,
                inner: inner.clone(),
                is_mut: true,
                is_unique: false,
            },
        ),
        PointerOperation::Read
        | PointerOperation::ReadUnaligned
        | PointerOperation::ReadVolatile => (vec![], (**inner).clone()),
        PointerOperation::Write
        | PointerOperation::WriteUnaligned
        | PointerOperation::WriteVolatile
            if *is_mut =>
        {
            (vec![(**inner).clone()], Type::Void)
        }
        _ => return None,
    })
}
