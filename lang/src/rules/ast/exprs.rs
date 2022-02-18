use crate::rules::types::RangeType;

pub struct Location {
    row: usize,
    column: usize
}

impl Location {
    #[inline]
    pub fn new(row: u32, column: usize) -> Location {
        Location { row: row as usize, column }
    }
    #[inline]
    pub fn row(&self) -> usize { self.row }
    #[inline]
    pub fn column(&self) -> usize { self.column }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BinaryOperator {
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
pub enum UnaryOperator {
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

pub struct RuleExpr {
    name: String,
    when: Expr,
    parameters: Option<Vec<Expr>>,
    block: BlockExpr,
    location: Location,
}

impl RuleExpr {
    pub fn new(name: String, when: Expr, block: BlockExpr, location: Location) -> RuleExpr {
        Self::new_with_parameters(name, when, None, block, location)
    }

    pub fn new_with_parameters(name: String, when: Expr, parameters: Option<Vec<Expr>>, block: BlockExpr, location: Location) -> RuleExpr {
        RuleExpr { name, when, parameters, block, location }
    }

    #[inline]
    pub fn name(&self) -> &str { &self.name }
    #[inline]
    pub fn when(&self) -> &Expr { &self.when }
    #[inline]
    pub fn block(&self) -> &BlockExpr { &self.block }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }
    #[inline]
    pub fn parameters(&self) -> Option<&[Expr]> { self.parameters.map(|v| &v) }
}

pub struct WhenExpr {
    when: Expr,
    block: BlockExpr,
    location: Location,
}

impl WhenExpr {
    pub fn new(when: Expr, block: BlockExpr, location: Location) -> Self {
        WhenExpr { when, block, location }
    }

    #[inline]
    pub fn when(&self) -> &Expr { &self.when }
    #[inline]
    pub fn block(&self) -> &BlockExpr { &self.block }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

pub struct ArrayExpr {
    elements: Vec<Expr>,
    location: Location,
}

impl ArrayExpr {
    pub fn new(elements: Vec<Expr>, location: Location) -> ArrayExpr {
        ArrayExpr { elements, location }
    }

    #[inline]
    pub fn element(&self) -> &[Expr] { &self.elements }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

pub struct BlockExpr {
    assignments: Vec<LetExpr>,
    clause: Expr,
    location: Location,
}

impl BlockExpr {
    pub fn new(assignments: Vec<LetExpr>, clause: Expr, location: Location) -> BlockExpr {
        BlockExpr { assignments, clause,  location }
    }

    #[inline]
    pub fn assignments(&self) -> &[LetExpr] { &self.assignments }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }
    #[inline]
    pub fn clause(&self) -> &Expr { &self.clause }
}

pub struct LetExpr {
    name: String,
    value: Expr,
    location: Location,
}

impl LetExpr {
    pub fn new(name: String, value: Expr, location: Location) -> LetExpr {
        LetExpr { name, value, location }
    }

    #[inline]
    pub fn value(&self) -> &Expr { &self.value }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }
    #[inline]
    pub fn name(&self) -> &str { &self.name }
}

pub struct QueryExpr {
    parts: Vec<Expr>,
    location: Location,
}

impl QueryExpr {
    pub fn new(parts: Vec<Expr>, location: Location) -> QueryExpr {
        QueryExpr { parts, location }
    }

    #[inline]
    pub fn parts(&self) -> &[Expr] { &self.parts }

    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

pub struct TypeExpr {
    name: String,
    block: BlockExpr,
    location: Location,
}

impl TypeExpr {
    pub fn new(name: String, block: BlockExpr, location: Location) -> Self {
        TypeExpr { name, block, location }
    }

    #[inline]
    pub fn name(&self) -> &str { &self.name }

    #[inline]
    pub fn block(&self) -> &BlockExpr { &self.block }

    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

pub struct BinaryExpr {
    operator: BinaryOperator,
    lhs: Expr,
    rhs: Expr,
    location: Location,
}

impl BinaryExpr {
    pub fn new(operator: BinaryOperator, lhs: Expr, rhs: Expr, location: Location) -> BinaryExpr {
        BinaryExpr { lhs, rhs, operator, location }
    }

    #[inline]
    pub fn lhs(&self) -> &Expr { &self.lhs }

    #[inline]
    pub fn rhs(&self) -> &Expr { &self.rhs }

    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}



pub struct UnaryExpr {
    operator: UnaryOperator,
    expr: Expr,
    location: Location,
}

impl UnaryExpr {
    pub fn new(operator: UnaryOperator, expr: Expr, location: Location) -> Self {
        UnaryExpr { operator, expr, location }
    }

    #[inline]
    pub fn op(&self) -> UnaryOperator { self.operator }

    #[inline]
    pub fn expr(&self) -> &Expr { &self.expr }
}

pub struct MapExpr {
    entries: indexmap::IndexMap<String, Expr>,
    location: Location,
}

impl MapExpr {
    pub fn new(entries: indexmap::IndexMap<String, Expr>, location: Location) -> Self {
        MapExpr { entries, location }
    }

    #[inline]
    pub fn entries(&self) -> &indexmap::IndexMap<String, Expr> { &self.entries }

    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

pub struct StringExpr {
    value: String,
    location: Location
}

impl StringExpr {
    pub fn new(value: String, location: Location) -> Self {
        StringExpr { value, location }
    }

    #[inline]
    pub fn value(&self) -> &str { &self.value }

    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

pub struct RegexExpr {
    value: String,
    location: Location
}

impl RegexExpr {
    pub fn new(value: String, location: Location) -> Self {
        RegexExpr { value, location }
    }

    #[inline]
    pub fn value(&self) -> &str { &self.value }

    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

pub struct BoolExpr {
    value: bool,
    location: Location
}

impl BoolExpr {
    pub fn new(value: bool, location: Location) -> Self {
        BoolExpr { value, location }
    }

    #[inline]
    pub fn value(&self) -> bool { self.value }

    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

pub struct IntExpr {
    value: i64,
    location: Location
}

impl IntExpr {
    pub fn new(value: i64, location: Location) -> Self {
        IntExpr { value, location }
    }

    #[inline]
    pub fn value(&self) -> i64 { self.value }

    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

pub struct FloatExpr {
    value: f64,
    location: Location
}

impl FloatExpr {
    pub fn new(value: f64, location: Location) -> Self {
        FloatExpr { value, location }
    }

    #[inline]
    pub fn value(&self) -> f64 { self.value }

    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

pub struct CharExpr {
    value: char,
    location: Location
}

impl CharExpr {
    pub fn new(value: char, location: Location) -> Self {
        CharExpr { value, location }
    }

    #[inline]
    pub fn value(&self) -> char { self.value }

    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

pub struct RangeIntExpr {
    value: RangeType<i64>,
    location: Location
}

impl RangeIntExpr {
    pub fn new(value: RangeType<i64>, location: Location) -> Self {
        RangeIntExpr { value, location }
    }

    #[inline]
    pub fn value(&self) -> &RangeType<i64> { &self.value }

    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

pub struct RangeFloatExpr {
    value: RangeType<i64>,
    location: Location
}

impl RangeFloatExpr {
    pub fn new(value: RangeType<i64>, location: Location) -> Self {
        RangeFloatExpr { value, location }
    }

    #[inline]
    pub fn value(&self) -> &RangeType<i64> { &self.value }

    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}


//
// This is keeping a consistent memory profile for the Enum
//
pub enum Expr {
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
