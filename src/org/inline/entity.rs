use super::read::{Read, Stop};
use super::{InlineKind, Raw};

// Standard Org entity names are syntax data; no reference parser code is used.
const NAMES: &str = "Agrave agrave Aacute aacute Acirc acirc Amacr amacr Atilde atilde Auml auml Aring AA aring AElig aelig Ccedil ccedil Egrave egrave Eacute eacute Ecirc ecirc Euml euml Igrave igrave Iacute iacute Idot inodot Icirc icirc Iuml iuml Ntilde ntilde Ograve ograve Oacute oacute Ocirc ocirc Otilde otilde Ouml ouml Oslash oslash OElig oelig Scaron scaron szlig Ugrave ugrave Uacute uacute Ucirc ucirc Uuml uuml Yacute yacute Yuml yuml fnof real image weierp ell imath jmath Alpha alpha Beta beta Gamma gamma Delta delta Epsilon epsilon varepsilon Zeta zeta Eta eta Theta theta thetasym vartheta Iota iota Kappa kappa Lambda lambda Mu mu nu Nu Xi xi Omicron omicron Pi pi Rho rho Sigma sigma sigmaf varsigma Tau Upsilon upsih upsilon Phi phi varphi Chi chi acutex Psi psi tau Omega omega piv varpi partial alefsym aleph gimel beth dalet ETH eth THORN thorn dots cdots hellip middot iexcl iquest shy ndash mdash quot acute ldquo rdquo bdquo lsquo rsquo sbquo laquo raquo lsaquo rsaquo circ vert vbar brvbar S sect P para amp lt gt tilde slash plus under equal asciicirc dagger dag Dagger ddag nbsp ensp emsp thinsp curren cent pound yen euro EUR dollar USD copy reg trade minus pm plusmn times frasl colon div frac12 frac14 frac34 permil sup1 sup2 sup3 radic sum prod micro macr deg prime Prime infin infty prop propto not neg land wedge lor vee cap cup smile frown int therefore there4 because sim cong simeq asymp approx ne neq equiv triangleq le leq ge geq lessgtr lesseqgtr ll Ll lll gg Gg ggg prec preceq preccurlyeq succ succeq succcurlyeq sub subset sup supset nsub sube nsup supe setminus forall exist exists nexist nexists empty emptyset isin in notin ni nabla ang angle perp parallel sdot cdot lceil rceil lfloor rfloor lang rang langle rangle hbar mho larr leftarrow gets lArr Leftarrow uarr uparrow uArr Uparrow rarr to rightarrow rArr Rightarrow darr downarrow dArr Downarrow harr leftrightarrow hArr Leftrightarrow crarr hookleftarrow arccos arcsin arctan arg cos cosh cot coth csc deg det dim exp gcd hom inf ker lg lim liminf limsup ln log max min Pr sec sin sinh sup tan tanh bull bullet star lowast ast odot oplus otimes check checkmark ordf ordm cedil oline uml zwnj zwj lrm rlm smiley blacksmile sad frowny clubs clubsuit spades spadesuit hearts heartsuit diams diamondsuit diamond Diamond loz";

pub(super) fn recognize(read: &mut Read<'_>, start: usize) -> Result<Option<Raw>, Stop> {
    if read.byte(start + 1)? == Some(b'_') {
        return space(read, start);
    }
    let mut found = None;
    for name in NAMES.split_ascii_whitespace() {
        if let Some(node) = named(read, start, name)?
            && found
                .as_ref()
                .is_none_or(|old: &Raw| old.source.end < node.source.end)
        {
            found = Some(node);
        }
    }
    Ok(found)
}

fn named(read: &mut Read<'_>, start: usize, name: &str) -> Result<Option<Raw>, Stop> {
    if !read.starts(start + 1, name.as_bytes())? {
        return Ok(None);
    }
    let end = start + 1 + name.len();
    let braces = read.starts(end, b"{}")?;
    if !braces && read.character(end)?.is_some_and(char::is_alphabetic) {
        return Ok(None);
    }
    let tail = if braces { end + 2 } else { end };
    Ok(Some(Raw::new(
        InlineKind::Entity,
        start..tail,
        start + 1..end,
    )))
}

fn space(read: &mut Read<'_>, start: usize) -> Result<Option<Raw>, Stop> {
    let mut end = start + 2;
    while end < start + 22 && read.byte(end)? == Some(b' ') {
        end += 1;
    }
    if end == start + 2 {
        return Ok(None);
    }
    Ok(Some(Raw::new(
        InlineKind::Entity,
        start..end,
        start + 1..end,
    )))
}
