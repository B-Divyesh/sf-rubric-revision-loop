# Rubric Revision Loop — visual thesis

## Direction: a paper-cut revision desk

The interface is a small paper-cut diorama: an assignment is a physical sheet,
rubric codes are clipped tabs, and each revision moves a paper strip from the
teacher's desk to the student's workbench and back to the review tray. This is
specific to the product's job: making the *path between a reason and a revision*
visible. It avoids both institutional LMS chrome and the synthetic glow common
to AI writing products. Decoration is reserved for explaining that loop.

The product uses one explicitly painted warm-light treatment rather than a
theme switch. A dark mode would turn the paper metaphor into a screen metaphor
and weaken the visual thesis. Contrast is checked within this treatment.

## Palette

| Token | Value | Use |
| --- | --- | --- |
| Desk | `#F3E9D4` | page ground, unbleached paper |
| Paper | `#FFFDF7` | primary working surface |
| Ink | `#202824` | body and headings |
| Pencil | `#59655F` | secondary text (4.9:1 on paper) |
| Moss | `#176B4B` | primary actions, selected states |
| Moss dark | `#0E4A34` | hover/focus contrast |
| Chalk | `#FFFFFF` | text on moss |
| Marigold | `#E5A72D` | highlights and “awaiting” tabs |
| Rust | `#A33C2E` | destructive/error states |
| Sky paper | `#DCEBF2` | student-side layers |
| Leaf paper | `#DDE9D5` | completed/reviewed layers |
| Hairline | `#CEC3AD` | rules and input borders |

Shadows use warm transparent ink (`rgba(55, 45, 28, .14)`), offset rather than
blurred into glossy cards. State always includes icon/label, never color alone.

## Type

- Display and section headings: Georgia, Cambria, `Times New Roman`, serif. Its
  editorial cadence belongs to marked-up writing without needing a font
  download.
- Interface and body: Inter-compatible system stack (`ui-sans-serif`,
  `system-ui`, Segoe UI, sans-serif). Plain and highly legible for dense review
  work.
- Scale: 14 (metadata), 16 (body), 20 (label heading), 26 (section), 42–56
  (single page h1). Body leading is 1.55 and prose measure is capped at 68ch.

No external fonts are requested at runtime.

## Spacing and shape

An 8px base rhythm with 4px for fine alignment: 4, 8, 12, 16, 24, 32, 48,
64. Working areas use asymmetric paper edges and 2–8px corner radii rather
than uniformly rounded cards. Independent sheets use a 1px hairline and a
4px/7px offset shadow. Controls are at least 44px high, with 12px between
adjacent actions. Desktop uses a 268px navigation rail and broad desk; at
390px it becomes a compact top bar and all two-column forms stack.

## Interaction grammar

- Rubric codes behave like clipped paper tabs: selected tabs shift by 2px and
  reveal a check mark.
- Creating a feedback link slides a pale-blue student slip upward from the
  composer; a copy action changes its label to “Copied”.
- Queue rows are stacked sheets. Expanding one reveals the before/after pair in
  the same physical location, making the change easy to compare.
- Destructive actions name their target and require confirmation. Network
  results are announced in one polite live region.
- URLs are text fields beside explicit copy buttons; no icon-only actions.

## Motion

Interface transitions last 180–240ms and animate only opacity and transform.
New sheets rise 8px from their source; selected tabs depress 2px; queue details
fade in. Nothing loops. Under `prefers-reduced-motion: reduce`, transitions and
smooth scrolling are removed and every state change is instantaneous while
depth remains through borders, overlap, and shadows.

## Asset plan and provenance

The hero is one original wide raster illustration: a top-down handcrafted
paper diorama where a green rubric tab travels across a folded path from a
teacher's annotated page to a student's before/after revision and into a review
tray. It contains no people, text, logos, grades, screens, or AI imagery. It is
supporting explanation, not a claim that the product reads or writes work.

Prompt sheet:

> Use case: stylized-concept. Asset type: wide landing/workspace header
> illustration. Primary request: a refined handcrafted paper-cut diorama of a
> writing revision loop viewed at a gentle top-down angle. Scene: cream paper
> desk; at left an annotated manuscript with abstract pencil lines and a moss
> green rubric tab; a folded paper path curves through the center; at right a
> pale blue before/after excerpt sheet with one cut-paper arrow leading to a
> shallow leaf-green review tray. Style: tactile layered paper, deckled edges,
> subtle fibers, precise editorial illustration, restrained and calm.
> Composition: wide 3:2 landscape, subjects centered with breathing room,
> readable at small size. Lighting: soft window light from upper left, short
> warm shadows. Palette: unbleached cream, charcoal ink, moss green, pale blue,
> marigold accent, muted terracotta. Constraints: no humans, no hands, no
> letters, no legible text, no numbers, no logos, no watermark, no interface
> mockup, no photoreal stationery brands. Avoid: glossy 3D, gradients, neon,
> clutter, floating objects, generic corporate vectors, AI motifs.

Generation: Azure AI Foundry factory image deployment via the factory-provided
`/opt/fleet/lib/gen-image.sh`; generated 2026-08-27. The chosen PNG source and
prompt sidecar live in `assets/src/`; production WebP derivatives are optimized
under `frontend/public/assets/`. The generated asset is original to this
product. The footer discloses that the illustration was AI-generated.
