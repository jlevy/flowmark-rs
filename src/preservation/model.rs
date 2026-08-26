//! Typed records and fail-closed invariants for preserved source regions.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreservationError(pub(crate) &'static str);

impl fmt::Display for PreservationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for PreservationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RegionKind {
    MathGitlabInline,
    MathMystInline,
    RawHtmlInline,
    CodeSpan,
    MathParenInline,
    MathEnvironmentInline,
    AttributeGroupInline,
    MathDollarInline,
    MathDoubleDollarInline,
    PandocMultilineTable,
    ObsidianCallout,
    ColonContainer,
    TomlFrontmatter,
    DefinitionList,
    PandocGridTable,
    RawHtmlBlock,
    AttributeGroupBlock,
    MathDollarBlock,
    MathBracketBlock,
    MathEnvironmentBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegionForm {
    Inline,
    Block,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ContainerContext {
    pub(crate) blockquote_depth: usize,
    pub(crate) list_depth: usize,
    pub(crate) content_column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NormalizedSource {
    pub(crate) text: String,
    pub(crate) had_bom: bool,
}

impl NormalizedSource {
    pub(crate) fn byte_length(&self) -> usize {
        self.text.len()
    }

    pub(crate) fn slice(&self, start: usize, end: usize) -> Result<&str, PreservationError> {
        if start >= end || end > self.text.len() {
            return Err(PreservationError("preserved region is outside normalized source"));
        }
        self.text
            .get(start..end)
            .ok_or(PreservationError("preserved region is not on UTF-8 boundaries"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Candidate {
    pub(crate) kind: RegionKind,
    pub(crate) form: RegionForm,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) container: ContainerContext,
    pub(crate) scaffold_prefix: String,
}

impl Candidate {
    pub(crate) fn inline(kind: RegionKind, start: usize, end: usize) -> Self {
        Self {
            kind,
            form: RegionForm::Inline,
            start,
            end,
            container: ContainerContext::default(),
            scaffold_prefix: String::new(),
        }
    }

    pub(crate) fn block(
        kind: RegionKind,
        start: usize,
        end: usize,
        container: ContainerContext,
        scaffold_prefix: String,
    ) -> Self {
        Self { kind, form: RegionForm::Block, start, end, container, scaffold_prefix }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProtectedRegion {
    pub(crate) index: usize,
    pub(crate) kind: RegionKind,
    pub(crate) form: RegionForm,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) source: String,
    pub(crate) logical_widths: Vec<usize>,
    pub(crate) container: ContainerContext,
    pub(crate) scaffold_prefix: String,
}

impl ProtectedRegion {
    pub(crate) fn from_candidate(
        source: &NormalizedSource,
        candidate: Candidate,
        index: usize,
    ) -> Result<Self, PreservationError> {
        let exact = source.slice(candidate.start, candidate.end)?.to_owned();
        let logical_widths = if candidate.form == RegionForm::Inline {
            exact.split('\n').map(str::chars).map(Iterator::count).collect()
        } else {
            Vec::new()
        };
        if candidate.form == RegionForm::Block {
            if !exact.ends_with('\n') {
                return Err(PreservationError("protected block must end with LF"));
            }
            if !exact.starts_with(&candidate.scaffold_prefix) {
                return Err(PreservationError("block scaffold does not match source"));
            }
        }
        Ok(Self {
            index,
            kind: candidate.kind,
            form: candidate.form,
            start: candidate.start,
            end: candidate.end,
            source: exact,
            logical_widths,
            container: candidate.container,
            scaffold_prefix: candidate.scaffold_prefix,
        })
    }
}

pub(crate) fn validate_regions(
    source: &NormalizedSource,
    regions: &[ProtectedRegion],
) -> Result<(), PreservationError> {
    let mut previous_end = 0;
    for (expected_index, region) in regions.iter().enumerate() {
        if region.index != expected_index || region.start < previous_end {
            return Err(PreservationError("protected regions are not canonical source order"));
        }
        if source.slice(region.start, region.end)? != region.source {
            return Err(PreservationError("protected region does not match source"));
        }
        previous_end = region.end;
    }
    Ok(())
}
