use crate::types::{
    RangeType,
    Location
};
use serde::{Serialize, Deserialize};

/// AST Expressions for Guard Language
//#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Debug, Serialize, Deserialize)]
pub enum Expr {
    /// File that contains a list of rules that needs to be evaluated
    ///
    File(Box<FileExpr>),

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

    /// RuleClause used as a part of referring to named rule as a part a
    /// conjunction. This allows to write complex rules that can combined
    /// together a major objects. As an example
    ///
    /// # Examples
    ///
    /// rule encryption_at_rest {
    ///     s3_encryption_at_rest
    ///     ebs_volume_encryption_at_rest
    ///     ddb_encryption_at_rest
    ///     databases_encryption_at_rest
    /// }
    ///
    /// rule s3_encryption_at_rest {
    ///     s3_encrypt_server_side or s3_encrypt_with_kms
    ///     s3_policy_allow_only_encrypted_puts
    /// }
    ///
    RuleClause(Box<RuleClauseExpr>),

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
    /// A filter expression can be be one of 3 forms that is reflected in the [Expr]
    /// captured
    ///
    ///   Resources[ Type == /S3/ ] # actual clause form, this filter by the clause name
    ///   Resources[ %names ] # variable reference form, filters keys by the reference contained
    ///
    ///   Resources[ name | Type == /S3/ ]
    ///   Resources[ name | %name == /^not/ ] # filter keys that start with name
    ///   Resources[ name ]  # just capture variable.
    ///
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

    /// Variable definition
    ///
    Variable(Box<StringExpr>),

    /// Variable Reference
    ///
    VariableReference(Box<StringExpr>),

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Equals,
    NotEquals,
    Greater,
    GreaterThanEquals,
    Lesser,
    LesserThanEquals,
    In,
    NotIn,
    Add,
    Or,
    And
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
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
    NotExists,
    NotEmpty,
    IsNotString,
    IsNotList,
    IsNotMap,
    IsNotInt,
    IsNotFloat,
    IsNotBool,
    IsNotRegex,
    Not,
    Any,
    AnyOne,
    Keys,
    Indices
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileExpr {
    pub name: String,
    pub assignments: Vec<LetExpr>,
    pub rules: Vec<RuleExpr>,
    pub location: Location,
}

impl FileExpr {
    pub fn new(name: String, assignments: Vec<LetExpr>, rules: Vec<RuleExpr>) -> Self {
        FileExpr { name, assignments, rules, location: Location::new(1, 1) }
    }

    #[inline]
    pub fn name(&self) -> &str { &self.name }
    #[inline]
    pub fn assignments(&self) -> &[LetExpr] { &self.assignments }
    #[inline]
    pub fn rules(&self) -> &[RuleExpr] { &self.rules }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleExpr {
    pub name: String,
    pub when: Option<Expr>,
    pub parameters: Option<Vec<Expr>>,
    pub block: BlockExpr,
    pub location: Location,
}

impl RuleExpr {

    pub fn new(name: String, block: BlockExpr, location: Location) -> RuleExpr {
        Self::new_with_when(name, None, block, location)
    }

    pub fn new_with_when(name: String, when: Option<Expr>, block: BlockExpr, location: Location) -> RuleExpr {
        Self::new_with_when_parameters(name, when, None, block, location)
    }

    pub fn new_with_when_parameters(name: String, when: Option<Expr>, parameters: Option<Vec<Expr>>, block: BlockExpr, location: Location) -> RuleExpr {
        RuleExpr { name, when, parameters, block, location }
    }

    #[inline]
    pub fn name(&self) -> &str { &self.name }
    #[inline]
    pub fn when(&self) -> Option<&Expr>{ self.when.as_ref() }
    #[inline]
    pub fn block(&self) -> &BlockExpr { &self.block }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }
    #[inline]
    pub fn parameters(&self) -> Option<&[Expr]> { self.parameters.as_ref().map(Vec::as_slice) }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleClauseExpr {
    pub name: String,
    pub parameters: Option<Vec<Expr>>,
    pub location: Location,
    pub message: Option<String>,
}

impl RuleClauseExpr {
    pub fn new(name: String,
               parameters: Option<Vec<Expr>>,
               location: Location,
               message: Option<String>) -> Self {
        RuleClauseExpr {
            name,
            parameters,
            location,
            message
        }
    }

    #[inline]
    pub fn name(&self) -> &str { &self.name }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }
    #[inline]
    pub fn parameters(&self) -> Option<&[Expr]> { self.parameters.as_ref().map(Vec::as_slice) }
    #[inline]
    pub fn message(&self) -> Option<&str> { self.message.as_ref().map(String::as_str) }

}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhenExpr {
    pub when: Expr,
    pub block: BlockExpr,
    pub location: Location,
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

#[derive(Debug,  Serialize, Deserialize)]
pub struct ArrayExpr {
    pub elements: Vec<Expr>,
    pub location: Location,
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

#[derive(Debug,  Serialize, Deserialize)]
pub struct BlockExpr {
    pub assignments: Vec<Expr>,
    pub clause: Expr,
    pub location: Location,
}

impl BlockExpr {
    pub fn new(assignments: Vec<Expr>, clause: Expr, location: Location) -> BlockExpr {
        BlockExpr { assignments, clause,  location }
    }

    #[inline]
    pub fn assignments(&self) -> &[Expr] { &self.assignments }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }
    #[inline]
    pub fn clause(&self) -> &Expr { &self.clause }
}

#[derive(Debug,  Serialize, Deserialize)]
pub struct BlockClauseExpr {
   pub select: QueryExpr,
   pub block: BlockExpr,
   pub message: Option<String>,
   pub location: Location,
}

impl BlockClauseExpr {
    pub fn new(select: QueryExpr, block: BlockExpr, location: Location) -> Self {
        Self::new_with_msg(select, block, location, None)
    }

    pub fn new_with_msg(select: QueryExpr, block: BlockExpr, location: Location, message: Option<String>) -> Self {
        BlockClauseExpr { select, block, location, message }
    }

    #[inline]
    pub fn select(&self) -> &QueryExpr { &self.select }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }
    #[inline]
    pub fn clause(&self) -> &BlockExpr { &self.block }
    #[inline]
    pub fn message(&self) -> Option<&str> { self.message.as_ref().map(String::as_str) }
}

#[derive(Debug,  Serialize, Deserialize)]
pub struct LetExpr {
   pub key: Option<String>,
   pub name: String,
   pub value: Expr,
   pub location: Location,
}

impl LetExpr {
    pub fn new(name: String, value: Expr, location: Location) -> LetExpr {
        Self::new_with_key(None, name, value, location)
    }

    pub fn new_with_key(key: Option<String>, name: String, value: Expr, location: Location) -> LetExpr {
        LetExpr { key, name, value, location }
    }

    #[inline]
    pub fn value(&self) -> &Expr { &self.value }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }
    #[inline]
    pub fn name(&self) -> &str { &self.name }
}

#[derive(Debug,  Serialize, Deserialize)]
pub struct QueryExpr {
   pub parts: Vec<Expr>,
   pub location: Location,
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

#[derive(Debug,  Serialize, Deserialize)]
pub struct BinaryExpr {
    pub operator: BinaryOperator,
    pub lhs: Expr,
    pub rhs: Expr,
    pub message: Option<String>,
    pub location: Location,
}

impl BinaryExpr {
    pub fn new(operator: BinaryOperator, lhs: Expr, rhs: Expr, location: Location) -> BinaryExpr {
        Self::new_with_msg(operator, lhs, rhs, location, None)
    }

    pub fn new_with_msg(operator: BinaryOperator, lhs: Expr, rhs: Expr, location: Location, message: Option<String>) -> BinaryExpr {
        BinaryExpr { lhs, rhs, operator, location, message }
    }

    #[inline]
    pub fn lhs(&self) -> &Expr { &self.lhs }
    #[inline]
    pub fn rhs(&self) -> &Expr { &self.rhs }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }
    #[inline]
    pub fn message(&self) -> Option<&str> { self.message.as_ref().map(String::as_str) }

}

#[derive(Debug,  Serialize, Deserialize)]
pub struct UnaryExpr {
    pub operator: UnaryOperator,
    pub expr: Expr,
    pub message: Option<String>,
    pub location: Location,
}

impl UnaryExpr {
    pub fn new(operator: UnaryOperator, expr: Expr, location: Location) -> Self {
        Self::new_with_msg(operator, expr, location, None)
    }

    pub fn new_with_msg(operator: UnaryOperator, expr: Expr, location: Location, message: Option<String>) -> Self {
        UnaryExpr { operator, expr, location, message }
    }

    #[inline]
    pub fn op(&self) -> UnaryOperator { self.operator }
    #[inline]
    pub fn expr(&self) -> &Expr { &self.expr }
    #[inline]
    pub fn message(&self) -> Option<&str> { self.message.as_ref().map(String::as_str) }
    #[inline]
    pub fn location(&self) -> &Location { &self.location }

}

#[derive(Debug,  Serialize, Deserialize)]
pub struct MapExpr {
    pub entries: indexmap::IndexMap<String, Expr>,
    pub location: Location,
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

#[derive(Debug,  Hash, Serialize, Deserialize)]
pub struct StringExpr {
    pub value: String,
    pub location: Location
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

#[derive(Debug,  Serialize, Deserialize)]
pub struct RegexExpr {
    pub value: String,
    pub location: Location
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

#[derive(Debug,  Serialize, Deserialize)]
pub struct BoolExpr {
    pub value: bool,
    pub location: Location
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

#[derive(Debug,  Serialize, Deserialize)]
pub struct IntExpr {
    pub value: i64,
    pub location: Location
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

#[derive(Debug, Serialize, Deserialize)]
pub struct FloatExpr {
    pub value: f64,
    pub location: Location
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

#[derive(Debug,  Serialize, Deserialize)]
pub struct CharExpr {
    pub value: char,
    pub location: Location
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

#[derive(Debug,  Serialize, Deserialize)]
pub struct RangeIntExpr {
    pub value: RangeType<i64>,
    pub location: Location
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

#[derive(Debug, Serialize, Deserialize)]
pub struct RangeFloatExpr {
    pub value: RangeType<f64>,
    pub location: Location
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


impl Expr {
    pub fn accept<'expr, V>(&'expr self, visitor: V) -> Result<V::Value, V::Error>
        where
            V: crate::visitor::Visitor<'expr>
    {
        match self {
            Expr::File(value_expr) => visitor.visit_file(self, value_expr),
            Expr::Rule(value_expr) => visitor.visit_rule(self, value_expr),
            Expr::RuleClause(value_expr) => visitor.visit_rule_clause(self, value_expr),
            Expr::Let(value_expr) => visitor.visit_let(self, value_expr),
            Expr::When(value_expr) => visitor.visit_when(self, value_expr),
            Expr::Select(value_expr) => visitor.visit_select(self, value_expr),
            Expr::BinaryOperation(value_expr) => visitor.visit_binary_operation(self, value_expr),
            Expr::UnaryOperation(value_expr) => visitor.visit_unary_operation(self, value_expr),
            Expr::Array(value_expr) => visitor.visit_array(self, value_expr),
            Expr::Map(value_expr) => visitor.visit_map(self, value_expr),
            Expr::Null(value_expr) => visitor.visit_null(self, value_expr),
            Expr::String(value_expr) => visitor.visit_string(self, value_expr),
            Expr::Regex(value_expr) => visitor.visit_regex(self, value_expr),
            Expr::Char(value_expr) => visitor.visit_char(self, value_expr),
            Expr::Bool(value_expr) => visitor.visit_bool(self, value_expr),
            Expr::Int(value_expr) => visitor.visit_int(self, value_expr),
            Expr::Float(value_expr) => visitor.visit_float(self, value_expr),
            Expr::RangeInt(value_expr) => visitor.visit_range_int(self, value_expr),
            Expr::RangeFloat(value_expr) => visitor.visit_range_float(self, value_expr),
            Expr::Filter(value_expr) => visitor.visit_filter(self, value_expr),
            Expr::Variable(value_expr) => visitor.visit_variable(self, value_expr),
            Expr::VariableReference(value_expr) => visitor.visit_variable_reference(self, value_expr),
            Expr::Block(value_expr) => visitor.visit_block(self, value_expr),
        }
    }

    pub fn get_location(&self) -> &Location {
        match self {
            Expr::File(value_expr) => &value_expr.location,
            Expr::Rule(value_expr) =>  &value_expr.location,
            Expr::RuleClause(value_expr) => value_expr.location(),
            Expr::Let(value_expr) => &value_expr.location,
            Expr::When(value_expr) => &value_expr.location,
            Expr::Select(value_expr) => &value_expr.location,
            Expr::BinaryOperation(value_expr) => &value_expr.location,
            Expr::UnaryOperation(value_expr) => &value_expr.location,
            Expr::Array(value_expr) => &value_expr.location,
            Expr::Map(value_expr) => &value_expr.location,
            Expr::Null(value_expr) => &value_expr,
            Expr::String(value_expr) => &value_expr.location,
            Expr::Regex(value_expr) => &value_expr.location,
            Expr::Char(value_expr) => &value_expr.location,
            Expr::Bool(value_expr) => &value_expr.location,
            Expr::Int(value_expr) => &value_expr.location,
            Expr::Float(value_expr) => &value_expr.location,
            Expr::RangeInt(value_expr) => &value_expr.location,
            Expr::RangeFloat(value_expr) => &value_expr.location,
            Expr::Filter(value_expr) => &value_expr.location,
            Expr::Variable(value_expr) => &value_expr.location,
            Expr::VariableReference(value_expr) => &value_expr.location,
            Expr::Block(value_expr) => &value_expr.location,
        }
    }
}
