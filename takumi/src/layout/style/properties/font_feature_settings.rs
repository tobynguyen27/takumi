use parley::FontFeature;
use smallvec::SmallVec;

/// Controls OpenType font features via CSS font-feature-settings property.
///
/// This allows enabling/disabling specific typographic features in OpenType fonts
/// such as ligatures, kerning, small caps, and other advanced typography features.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FontFeatureSettings(pub SmallVec<[FontFeature; 4]>);
