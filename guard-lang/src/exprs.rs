use crate::types::{
    RangeType,
    Location
};

/// AST Expressions for Guard Language
#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    /// Rule expression in the language. Rules can be parameterized
    /// to create reusable rules. Rules are defined using the general form
    ///
    ///
    /// rule <name> <conditions> { clause-statements }
    ///
    /// # Examples
    ///
    /// rule s3_buckets {
    ///    Resources[ Types == 'AWS::S3::Bucket' ].Properties.BucketName == /^MyCompany/
    /// }
    Rule(Box<RuleExpr>),

    /// Let assignment statement for variable
    /// # Examples
    ///
    /// let s3_buckets = Resources[ Type == 'AWS::S3::Bucket' ]
    ///
    Let(Box<LetExpr>),

    /// A condition based block expression that evaluates the contexts on the block
    /// if the condition success. The context inside the block can reference anything
    /// defined inside and the outer scope of the block
    ///
    /// # Examples
    ///
    /// when %s3_buckets NOT EMPTY {
    ///    %s3_buckets.Properties.BucketName == /^MyCompany/
    /// }
    When(Box<WhenExpr>),

    /// Query Block expression selects all the entries for the query and then for each
    /// value from the query evaluates the list of clause expression provided in the
    /// accompanying block.
    ///
    /// # Examples
    ///
    /// Resources[ Type == 'AWS::S3::Bucket' ] {
    ///     Properties {
    ///        BucketName == /^MyCompany/
    ///        AccessControlList NOT EXISTS
    ///     }
    /// }
    ///
    /// A Type expression is a short cut way to expressing filtering of resources by
    /// type. The expression is equivalent of Resources[ Type == 'AWS::S3::Bucket' ]
    /// shown above.
    ///
    /// AWS::S3::Bucket { ... }
    ///
    Block(Box<BlockClauseExpr>),

    /// A dotted expression representing a query to select a set of values. Queries
    /// can be embedded filters on a collection or struct/map type. It can also contain
    /// references to variables as a part of the expression.
    ///
    /// # Examples
    ///
    /// Resources[ Type == /S3/ ].Properties.Tags[ Key == /^App/ ]
    /// Resources.local_s3_refs
    /// Resources.%local_s3_ref # explicitly referencing a variable
    ///
    Select(Box<QueryExpr>),

    /// A block expression that is effectively a part of a Query to filter down values
    ///
    Filter(Box<BlockExpr>),

    /// Binary Operation [BinaryOperator] when evaluating two expressions.
    ///
    /// # Examples
    ///
    /// Resources.*.Properties.Tags.*.Key != /Exception/
    ///
    BinaryOperation(Box<BinaryExpr>),

    /// Unary operation [UnaryOperator] against an expression result
    ///
    UnaryOperation(Box<UnaryExpr>),

    /// A collection Value object that has a mix of values and query expressions.
    ///
    /// # Examples
    ///
    /// let selected = [10, 20, Resources[ Type == /EC2::Instance/ ].Properties.Network.Ports[*] ]
    ///
    Array(Box<ArrayExpr>),

    /// A structured key value object that has a mix of values and query expressions
    ///
    /// # Examples
    ///
    Map(Box<MapExpr>),

    /// Sentinel value that means NULL
    ///
    Null(Box<Location>),

    /// A String literal value in the language or a Property value when used as a part of Query
    ///
    String(Box<StringExpr>),

    /// A regular expression liternal Value in the language
    ///
    Regex(Box<RegexExpr>),

    /// A boolean literal value in the language
    ///
    Bool(Box<BoolExpr>),

    /// A integer literal value in the language
    ///
    Int(Box<IntExpr>),

    /// A float literal value in the language
    ///
    Float(Box<FloatExpr>),

    /// A char literal value in the language
    Char(Box<CharExpr>),

    /// A range value expression that is use to denote a contiguous range of values
    ///
    /// # Examples
    ///
    /// let ports = r[80..800)
    ///
    RangeInt(Box<RangeIntExpr>),

    /// A range value expression that is use to denote a contiguous range of values
    ///
    /// # Examples
    ///
    /// let ports = r[0.8..1.4)
    ///
    RangeFloat(Box<RangeFloatExpr>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

#[derive(Debug, PartialEq, Eq)]
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
    pub fn parameters(&self) -> Option<&[Expr]> { self.parameters.as_ref().map(Vec::as_slice) }
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct BlockClauseExpr {
    select: QueryExpr,
    block: BlockExpr,
    location: Location,
}

impl BlockClauseExpr {
    pub fn new(select: QueryExpr, block: BlockExpr, location: Location) -> Self {
        BlockClauseExpr { select, block, location }
    }

    #[inline]
    pub fn select(&self) -> &QueryExpr { &self.select }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }
    #[inline]
    pub fn clause(&self) -> &BlockExpr { &self.block }
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct MapExpr {
    entries: indexmap::IndexMap<StringExpr, Expr>,
    location: Location,
}

impl MapExpr {
    pub fn new(entries: indexmap::IndexMap<StringExpr, Expr>, location: Location) -> Self {
        MapExpr { entries, location }
    }

    #[inline]
    pub fn entries(&self) -> &indexmap::IndexMap<StringExpr, Expr> { &self.entries }

    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

#[derive(Debug, PartialEq, Eq, Hash)]
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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug)]
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

impl PartialEq for FloatExpr {
    fn eq(&self, other: &Self) -> bool {
        self.value.partial_cmp(&other.value)
            .map_or(false, |ordering| match ordering {
                std::cmp::Ordering::Equal => true,
                _ => false
            })
    }
}

impl Eq for FloatExpr {}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug)]
pub struct RangeFloatExpr {
    value: RangeType<f64>,
    location: Location
}

impl RangeFloatExpr {
    pub fn new(value: RangeType<f64>, location: Location) -> Self {
        RangeFloatExpr { value, location }
    }

    #[inline]
    pub fn value(&self) -> &RangeType<f64> { &self.value }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }
}

impl PartialEq for RangeFloatExpr {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl Eq for RangeFloatExpr {}


