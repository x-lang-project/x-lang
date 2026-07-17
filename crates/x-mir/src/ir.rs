use x_ast::{Operator, Type, UnaryOperator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockIdentifier(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalIdentifier(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub enum Place {
    Local(LocalIdentifier),
    Global {
        name: String,
        value_type: Type,
    },
    Field {
        base: Box<Place>,
        field_name: String,
        value_type: Type,
    },
    Index {
        base: Box<Place>,
        index: Operand,
        value_type: Type,
    },
    Dereference {
        base: Box<Place>,
        value_type: Type,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Copy(Box<Place>),
    Move(Box<Place>),
    ConstantInteger(i64),
    ConstantFloat(f64),
    ConstantBoolean(bool),
    ConstantCharacter(char),
    ConstantString(String),
    TypeLiteral(Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Rvalue {
    Use(Operand),
    UnaryOperator(UnaryOperator, Operand),
    BinaryOperator(Operator, Operand, Operand),
    PointerOffset {
        base: Operand,
        offset: Operand,
    },
    Cast {
        expression: Operand,
        to: Type,
    },
    AddressOf {
        is_mut: bool,
        is_unique: bool,
        expression: Place,
    },
    Tuple(Vec<Operand>),
    Array(Vec<Operand>),
    SizeOf(Type),
    Call {
        function: String,
        arguments: Vec<Operand>,
    },

    TryCall {
        function: String,
        arguments: Vec<Operand>,
    },
    CatchCall {
        function: String,
        arguments: Vec<Operand>,
        fallback: Operand,
    },
    StructInstantiate {
        name: String,
        fields: Vec<(String, Operand)>,
    },
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum MirStatement {
    Assign(Place, Rvalue),
    StorageLive(LocalIdentifier),
    StorageDead(LocalIdentifier),
    Comment(String),
    ParallelFor {
        start: Operand,
        end: Operand,
        worker: String,
        captures: Vec<Operand>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirTerminator {
    Return,
    Goto(BlockIdentifier),
    SwitchInteger {
        discriminant: Operand,
        targets: Vec<(i64, BlockIdentifier)>,
        otherwise: BlockIdentifier,
    },
    Call {
        function: String,
        arguments: Vec<Operand>,
        destination: Place,
        target: BlockIdentifier,
    },
    TailCall {
        function: String,
        arguments: Vec<Operand>,
    },
    Throw {
        error: Operand,
    },
    TryCall {
        function: String,
        arguments: Vec<Operand>,
        destination: Place,
        target: BlockIdentifier,
    },
    CatchCall {
        function: String,
        arguments: Vec<Operand>,
        fallback: Operand,
        destination: Place,
        target: BlockIdentifier,
    },

    Abort,

    Exit {
        code: Operand,
    },
    Unreachable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    pub statements: Vec<MirStatement>,
    pub terminator: Option<MirTerminator>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalDeclaration {
    pub identifier: LocalIdentifier,
    pub name: Option<String>,
    pub value_type: Type,
}
