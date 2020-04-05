//! latex2mathml
//! 
//! Provides a functionility to convert LaTeX math equations to MathML representation.
//! This crate is implemented in pure Rust, so it works for all platforms including WebAssembly.
//! 
//! # Usage
//! 
//!  Main functions of this crate are  [`latex_to_mathml`](./fn.latex_to_mathml.html) and 
//! [`replace`](./fn.replace.html).
//! 
//! ```rust
//! use latex2mathml::{latex_to_mathml, DisplayStyle};
//! 
//! let latex = r#"\erf ( x ) = \frac{ 2 }{ \sqrt{ \pi } } \int_0^x e^{- t^2} \, dt"#;
//! let mathml = latex_to_mathml(latex, DisplayStyle::Block).unwrap();
//! println!("{}", mathml);
//! ```
//! 
//! For converting a document including LaTeX equasions, the function [`replace`](./fn.replace.html) 
//! may be useful.
//! 
//! ```rust
//! let latex = r#"The error function $\erf ( x )$ is defined by
//! $$\erf ( x ) = \frac{ 2 }{ \sqrt{ \pi } } \int_0^x e^{- t^2} \, dt .$$"#;
//! 
//! let mathml = latex2mathml::replace(latex).unwrap();
//! println!("{}", mathml);
//! ```
//! 
//! For more examples please check `examples/equations.rs` and `examples/document.rs`.
//! 

pub mod attribute;
pub mod token;
pub mod ast;
pub(crate) mod lexer;
pub(crate) mod parse;
mod error;
pub use error::LatexError;
use std::fmt;

/// display style
#[derive(Debug, Clone)]
pub enum DisplayStyle {
    Block,
    Inline
}

impl fmt::Display for DisplayStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplayStyle::Block => write!(f, "block"),
            DisplayStyle::Inline => write!(f, "inline"),
        }
    }
}

fn convert_content(latex: &str) -> Result<String, error::LatexError> {
    let l = lexer::Lexer::new(latex);
    let mut p = parse::Parser::new(l);
    let nodes = p.parse()?;

    let mathml = nodes.iter()
        .map(|node| format!("{}", node))
        .collect::<String>();
    
    Ok(mathml)
}

/// Convert LaTeX text to MathML.
/// 
/// The second argument specifies whether it is inline-equation or block-equation.
/// 
/// ```rust
/// use latex2mathml::{latex_to_mathml, DisplayStyle};
/// 
/// let latex = r#"(n + 1)! = \Gamma ( n + 1 )"#;
/// let mathml = latex_to_mathml(latex, DisplayStyle::Inline).unwrap();
/// println!("{}", mathml);
/// 
/// let latex = r#"x = \frac{ - b \pm \sqrt{ b^2 - 4 a c } }{ 2 a }"#;
/// let mathml = latex_to_mathml(latex, DisplayStyle::Block).unwrap();
/// println!("{}", mathml);
/// ```
/// 
pub fn latex_to_mathml(latex: &str, display: DisplayStyle) -> Result<String, error::LatexError> {
    let mathml = convert_content(latex)?;

    Ok(format!(
        r#"<math xmlns="http://www.w3.org/1998/Math/MathML" display="{}">{}</math>"#,
        display, mathml
    ))
}

/// Find LaTeX equations and replace them to MathML.
/// 
/// - inline-math: `$..$`
/// - display-math: `$$..$$`
/// 
/// Note that dollar signs that do not enclose a LaTeX equation (e.g. `This apple is $3.`) must not appear 
/// in the input string. For HTML, please use `&dollar;` instead of `$`.
/// 
/// ```rust
/// let input = r#"$E = m c^2$ is the most famous equation derived by Einstein.
/// In fact, this relation is a spacial case of the equation
/// $$E = \sqrt{ m^2 c^4 + p^2 c^2 } ,$$
/// which describes the relation between energy and momentum."#;
/// let output = latex2mathml::replace(input).unwrap();
/// println!("{}", output);
/// ```
/// 
/// `examples/document.rs` gives a sample code using this function.
/// 
pub fn replace(input: &str) -> Result<String, error::LatexError> {
    let mut input: Vec<u8> = input.as_bytes().to_owned();

    //**** Convert block-math ****//

    // `$$` に一致するインデックスのリストを生成
    let idx = input.windows(2).enumerate()
        .filter_map(|(i, window)| if window == &[b'$', b'$'] {
            Some(i)
        } else { None }).collect::<Vec<usize>>();
    if idx.len()%2 != 0 {
        return Err(LatexError::InvalidNumberOfDollarSigns);
    }

    if idx.len() > 1 {
        let mut output = Vec::new();
        output.extend_from_slice(&input[0..idx[0]]);
        for i in (0..idx.len()-1).step_by(2) {
            { // convert LaTeX to MathML
                let input = &input[idx[i]+2..idx[i+1]];
                let input = unsafe { std::str::from_utf8_unchecked(input) };
                let mathml = latex_to_mathml(input, DisplayStyle::Block)?;
                output.extend_from_slice(mathml.as_bytes());
            }

            if i+2 < idx.len() {
                output.extend_from_slice(&input[idx[i+1]+2..idx[i+2]]);
            } else {
                output.extend_from_slice(&input[idx.last().unwrap()+2..]);
            }
        }

        input = output;
    }

    //**** Convert inline-math ****//
    
    // `$` に一致するインデックスのリストを生成
    let idx = input.iter().enumerate()
        .filter_map(|(i, byte)| if byte == &b'$' {
            Some(i)
        } else { None }).collect::<Vec<usize>>();
    if idx.len()%2 != 0 {
        return Err(LatexError::InvalidNumberOfDollarSigns);
    }

    if idx.len() > 1 {
        let mut output = Vec::new();
        output.extend_from_slice(&input[0..idx[0]]);
        for i in (0..idx.len()-1).step_by(2) {
            { // convert LaTeX to MathML
                let input = &input[idx[i]+1..idx[i+1]];
                let input = unsafe { std::str::from_utf8_unchecked(input) };
                let mathml = latex_to_mathml(input, DisplayStyle::Inline)?;
                output.extend_from_slice(mathml.as_bytes());
            }

            if i+2 < idx.len() {
                output.extend_from_slice(&input[idx[i+1]+1..idx[i+2]]);
            } else {
                output.extend_from_slice(&input[idx.last().unwrap()+1..]);
            }
        }

        input = output;
    }

    unsafe {
        Ok(String::from_utf8_unchecked(input))
    }
}

#[cfg(test)]
mod tests {
    use super::convert_content;

    #[test]
    fn it_works() {
        let problems = vec![
            (r"0",            "<mn>0</mn>"),
            (r"3.14",         "<mn>3.14</mn>"),
            (r"x",            "<mi>x</mi>"),
            (r"\alpha",       "<mi>α</mi>"),
            (r"\phi/\varphi", "<mi>ϕ</mi><mo>/</mo><mi>φ</mi>"),
            (r"x = 3+\alpha", "<mi>x</mi><mo>=</mo><mn>3</mn><mo>+</mo><mi>α</mi>"),
            (r"\sin x",       "<mi>sin</mi><mi>x</mi>"),
            (r"\sqrt 2",      "<msqrt><mn>2</mn></msqrt>"),
            (r"\sqrt{x+2}",   "<msqrt><mrow><mi>x</mi><mo>+</mo><mn>2</mn></mrow></msqrt>"),
            (r"\sqrt[3]{x}",  "<mroot><mi>x</mi><mn>3</mn></mroot>"),
            (r"\frac{1}{2}",  "<mfrac><mn>1</mn><mn>2</mn></mfrac>"),
            (r"x^2",          "<msup><mi>x</mi><mn>2</mn></msup>"),
            (r"g_{\mu\nu}",   "<msub><mi>g</mi><mrow><mi>μ</mi><mi>ν</mi></mrow></msub>"),
            (r"\dot{x}",      "<mover><mi>x</mi><mo accent=\"true\">\u{02d9}</mo></mover>"),
            (r"\int dx",      r#"<mo>∫</mo><mi>d</mi><mi>x</mi>"#),
            (r"\oint_C dz",   r#"<msub><mo>∮</mo><mi>C</mi></msub><mi>d</mi><mi>z</mi>"#),
            (r"\overset{n}{X}", "<mover><mi>X</mi><mi>n</mi></mover>"),
            (r"\int_0^1 dx",  r#"<msubsup><mo>∫</mo><mn>0</mn><mn>1</mn></msubsup><mi>d</mi><mi>x</mi>"#),
            (r"\int^1_0 dx",  r#"<msubsup><mo>∫</mo><mn>0</mn><mn>1</mn></msubsup><mi>d</mi><mi>x</mi>"#),
            (r"\bm{x}",       r#"<mi mathvariant="bold-italic">x</mi>"#),
            (r"\mathbb{R}",   r#"<mi mathvariant="double-struck">R</mi>"#),
            (r"\sum_{i = 0}^∞ i", r#"<munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>0</mn></mrow><mi mathvariant="normal">∞</mi></munderover><mi>i</mi>"#),
            (r"\prod_n n",        r#"<munder><mo>∏</mo><mi>n</mi></munder><mi>n</mi>"#),
            (r"x\ y",         r#"<mi>x</mi><mspace width="1em"/><mi>y</mi>"#),
            (r"\left( x \right)", r#"<mrow><mo stretchy="true" form="prefix">(</mo><mi>x</mi><mo stretchy="true" form="postfix">)</mo></mrow>"#),
            (
                r"\left\{ x  ( x + 2 ) \right\}", 
                r#"<mrow><mo stretchy="true" form="prefix">{</mo><mrow><mi>x</mi><mo>(</mo><mi>x</mi><mo>+</mo><mn>2</mn><mo>)</mo></mrow><mo stretchy="true" form="postfix">}</mo></mrow>"#
            ),
            (r"f'", r#"<msup><mi>f</mi><mo>′</mo></msup>"#),
            (
                r"\begin{pmatrix} x \\ y \end{pmatrix}", 
                r#"<mrow><mo stretchy="true" form="prefix">(</mo><mtable><mtr><mtd><mi>x</mi></mtd></mtr><mtr><mtd><mi>y</mi></mtd></mtr></mtable><mo stretchy="true" form="postfix">)</mo></mrow>"#
            ),
        ];

        for (problem, answer) in problems.iter() {
            let mathml = convert_content(problem).unwrap();
            assert_eq!(&mathml, answer);
        }
    }
}