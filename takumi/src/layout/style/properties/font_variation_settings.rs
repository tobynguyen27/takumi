use parley::FontVariation;
use smallvec::SmallVec;

/// Controls variable font axis values via CSS font-variation-settings property.
///
/// This allows fine-grained control over variable font characteristics like weight,
/// width, slant, and other custom axes defined in the font.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FontVariationSettings(pub SmallVec<[FontVariation; 4]>);
