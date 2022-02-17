use crate::rules::types::RangeType;

pub(crate) struct Location {
    row: usize,
    column: usize
}

impl Location {
    #[inline]
    pub(crate) fn new(row: u32, column: usize) -> Location {
        Location { row: row as usize, column }
    }

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

#[derive(Clone, Copy, Eq, PartialEq)]
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

pub(crate) struct RuleExpr {
    name: String,
    when: Expr,
    parameters: Option<Vec<Expr>>,
    block: BlockExpr,
    location: Location,
}

impl RuleExpr {
    pub(crate) fn new(name: String, when: Expr, block: BlockExpr, location: Location) -> RuleExpr {
        Self::new_with_parameters(name, when, None, block, location)
    }

    pub(crate) fn new_with_parameters(name: String, when: Expr, parameters: Option<Vec<Expr>>, block: BlockExpr, location: Location) -> RuleExpr {
        RuleExpr { name, when, parameters, block, location }
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

pub(crate) struct WhenExpr {
    when: Expr,
    block: BlockExpr,
    location: Location,
}

impl WhenExpr {
    pub(crate) fn new(when: Expr, block: BlockExpr, location: Location) -> Self {
        WhenExpr { when, block, location }
    }

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
        BlockExpr { assignments, clause,  location }
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

impl QueryExpr {
    pub(crate) fn new(parts: Vec<Expr>, location: Location) -> QueryExpr {
        QueryExpr { parts, location }
    }

    #[inline]
    pub(crate) fn parts(&self) -> &[Expr] { &self.parts }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}

pub(crate) struct TypeExpr {
    name: String,
    block: BlockExpr,
    location: Location,
}

impl TypeExpr {
    pub(crate) fn new(name: String, block: BlockExpr, location: Location) -> Self {
        TypeExpr { name, block, location }
    }

    #[inline]
    pub(crate) fn name(&self) -> &str { &self.name }

    #[inline]
    pub(crate) fn block(&self) -> &BlockExpr { &self.block }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}

pub(crate) struct BinaryExpr {
    operator: BinaryOperator,
    lhs: Expr,
    rhs: Expr,
    location: Location,
}

impl BinaryExpr {
    pub(crate) fn new(operator: BinaryOperator, lhs: Expr, rhs: Expr, location: Location) -> BinaryExpr {
        BinaryExpr { lhs, rhs, operator, location }
    }

    #[inline]
    pub(crate) fn lhs(&self) -> &Expr { &self.lhs }

    #[inline]
    pub(crate) fn rhs(&self) -> &Expr { &self.rhs }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}



pub(crate) struct UnaryExpr {
    operator: UnaryOperator,
    expr: Expr,
    location: Location,
}

impl UnaryExpr {
    pub(crate) fn new(operator: UnaryOperator, expr: Expr, location: Location) -> Self {
        UnaryExpr { operator, expr, location }
    }

    #[inline]
    pub(crate) fn op(&self) -> UnaryOperator { self.operator }

    #[inline]
    pub(crate) fn expr(&self) -> &Expr { &self.expr }
}

pub(crate) struct MapExpr {
    entries: indexmap::IndexMap<String, Expr>,
    location: Location,
}

impl MapExpr {
    pub(crate) fn new(entries: indexmap::IndexMap<String, Expr>, location: Location) -> Self {
        MapExpr { entries, location }
    }

    #[inline]
    pub(crate) fn entries(&self) -> &indexmap::IndexMap<String, Expr> { &self.entries }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}

pub(crate) struct StringExpr {
    value: String,
    location: Location
}

impl StringExpr {
    pub(crate) fn new(value: String, location: Location) -> Self {
        StringExpr { value, location }
    }

    #[inline]
    pub(crate) fn value(&self) -> &str { &self.value }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}

pub(crate) struct RegexExpr {
    value: String,
    location: Location
}

impl RegexExpr {
    pub(crate) fn new(value: String, location: Location) -> Self {
        RegexExpr { value, location }
    }

    #[inline]
    pub(crate) fn value(&self) -> &str { &self.value }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}

pub(crate) struct BoolExpr {
    value: bool,
    location: Location
}

impl BoolExpr {
    pub(crate) fn new(value: bool, location: Location) -> Self {
        BoolExpr { value, location }
    }

    #[inline]
    pub(crate) fn value(&self) -> bool { self.value }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}

pub(crate) struct IntExpr {
    value: i64,
    location: Location
}

impl IntExpr {
    pub(crate) fn new(value: i64, location: Location) -> Self {
        IntExpr { value, location }
    }

    #[inline]
    pub(crate) fn value(&self) -> i64 { self.value }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}

pub(crate) struct FloatExpr {
    value: f64,
    location: Location
}

impl FloatExpr {
    pub(crate) fn new(value: f64, location: Location) -> Self {
        FloatExpr { value, location }
    }

    #[inline]
    pub(crate) fn value(&self) -> f64 { self.value }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}

pub(crate) struct CharExpr {
    value: char,
    location: Location
}

impl CharExpr {
    pub(crate) fn new(value: char, location: Location) -> Self {
        CharExpr { value, location }
    }

    #[inline]
    pub(crate) fn value(&self) -> char { self.value }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}

pub(crate) struct RangeIntExpr {
    value: RangeType<i64>,
    location: Location
}

impl RangeIntExpr {
    pub(crate) fn new(value: RangeType<i64>, location: Location) -> Self {
        RangeIntExpr { value, location }
    }

    #[inline]
    pub(crate) fn value(&self) -> &RangeType<i64> { &self.value }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}

pub(crate) struct RangeFloatExpr {
    value: RangeType<i64>,
    location: Location
}

impl RangeFloatExpr {
    pub(crate) fn new(value: RangeType<i64>, location: Location) -> Self {
        RangeFloatExpr { value, location }
    }

    #[inline]
    pub(crate) fn value(&self) -> &RangeType<i64> { &self.value }

    #[inline]
    pub(crate) fn location(&self) -> &Location { &self.location }
}


//
// This is keeping a consistent memory profile for the Enum
//
pub(crate) enum Expr {
    Rule(Box<RuleExpr>),
    Let(Box<LetExpr>),
    When(Box<WhenExpr>),
    Type(Box<TypeExpr>),
    Query(Box<QueryExpr>),
    BinaryOperation(Box<BinaryExpr>),
    UnaryOperation(Box<UnaryExpr>),
    Array(Box<ArrayExpr>),
    Map(Box<MapExpr>),
    Null(Box<Location>),
    String(Box<StringExpr>),
    Regex(Box<RegexExpr>),
    Bool(Box<BoolExpr>),
    Int(Box<IntExpr>),
    Float(Box<FloatExpr>),
    Char(Box<CharExpr>),
    RangeInt(Box<RangeIntExpr>),
    RangeFloat(Box<RangeFloatExpr>),
}
