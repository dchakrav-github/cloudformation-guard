use crate::rules::types::RangeType;

pub(crate) struct Location {
    row: usize,
    column: usize
}

impl Location {
    #[inline]
    pub(crate) fn row(&self) -> usize { self.row }

    #[inline]
    pub(crate) fn column(&self) -> usize { self.column }
}

pub(crate) enum BinaryOperator {
    Equals,
    NotEquals,
    Greater,
    GreaterThanEquals,
    Lesser,
    LesserThanEquals,
    In,
    Add,
    Or,
    And
}

pub(crate) enum UnaryOperator {
    Exists,
    Empty,
    IsString,
    IsList,
    IsMap,
    IsInt,
    IsFloat,
    IsBool,
    IsRegex,
    Not,
}

pub(crate) struct BinaryExpr {
    lhs: Expr,
    rhs: Expr
}

impl BinaryExpr {
    pub(crate) fn new(lhs: Expr, rhs: Expr) -> BinaryExpr {
        BinaryExpr { lhs, rhs }
    }

    #[inline]
    pub(crate) fn lhs(&self) -> &Expr { &self.lhs }

    #[inline]
    pub(crate) fn rhs(&self) -> &Expr { &self.rhs }
}

pub(crate) struct RuleExpr {
    name: String,
    when: Expr,
    block: BlockExpr,
    location: Location,
}

impl RuleExpr {
    pub(crate) fn new(name: String, when: Expr, block: BlockExpr, location: Location) -> RuleExpr {
        RuleExpr { name, when, block, location }
    }

    #[inline]
    pub(crate) fn name(&self) -> &str { &self.name }

    #[inline]
    pub(crate) fn when(&self) -> &Expr { &self.when }

    #[inline]
    pub(crate) fn block(&self) -> &BlockExpr { &self.block }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}

pub(crate) struct ArrayExpr {
    elements: Vec<Expr>,
    location: Location,
}

impl ArrayExpr {
    pub(crate) fn new(elements: Vec<Expr>, location: Location) -> ArrayExpr {
        ArrayExpr { elements, location }
    }

    #[inline]
    pub(crate) fn element(&self) -> &[Expr] { &self.elements }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}

pub(crate) struct BlockExpr {
    assignments: Vec<Expr>,
    clause: Expr,
    location: Location,
}

impl BlockExpr {
    pub(crate) fn new(assignments: Vec<Expr>, clause: Expr, location: Location) -> BlockExpr {
        BlockExpr { assignments, clause, location }
    }

    #[inline]
    pub(crate) fn assignments(&self) -> &[Expr] { &self.assignments }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }

    #[inline]
    pub(crate) fn clause(&self) -> &Expr { &self.clause }
}

pub(crate) struct LetExpr {
    name: String,
    value: Expr,
    location: Location,
}

impl LetExpr {
    pub(crate) fn new(name: String, value: Expr, location: Location) -> LetExpr {
        LetExpr { name, value, location }
    }

    #[inline]
    pub(crate) fn value(&self) -> &Expr { &self.value }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }

    #[inline]
    pub(crate) fn name(&self) -> &str { &self.name }
}

pub(crate) struct QueryExpr {
    parts: Vec<Expr>,
    location: Location,
}

impl ArrayExpr {
    pub(crate) fn new(parts: Vec<Expr>, location: Location) -> QueryExpr {
        QueryExpr { parts, location }
    }

    #[inline]
    pub(crate) fn parts(&self) -> &[Expr] { &self.parts }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}

pub(crate) enum Expr {
    Rule(RuleExpr),
    Let(LetExpr),
    Array(ArrayExpr),
    Null(Location),
    String(String, Location),
    Regex(String, Location),
    Bool(bool, Location),
    Int(i64, Location),
    Float(f64, Location),
    Char(char, Location),
    Map(indexmap::IndexMap<String, Expr>, Location),
    RangeInt(RangeType<i64>, Location),
    RangeFloat(RangeType<f64>, Location),
    RangeChar(RangeType<char>, Location),
    Query(QueryExpr),
    BinaryOperation(BinaryOperator, BinaryExpr),
    UnaryOperation(UnaryOperator, Expr)
}
