# BRUTALIST MONOCHROME DESIGN SYSTEM

## Philosophy

**Minimalist | Monospace | Geometric | Stark**

A brutalist design system built on the principles of:
- **Brutalism**: Raw, bold, geometric forms with no decoration
- **Monochrome**: Black, white, and grays only
- **Monospace**: Courier New/Monaco/Menlo fonts throughout
- **Minimalism**: Clean, functional, no unnecessary elements

## Color Palette

```css
--black: #000000      /* Primary text, borders, backgrounds */
--white: #FFFFFF      /* Background, inverse text */
--gray-100: #F5F5F5   /* Light backgrounds */
--gray-200: #E5E5E5   /* Borders, dividers */
--gray-300: #CCCCCC   /* Secondary text */
--gray-400: #999999   /* Muted text */
--gray-500: #666666   /* Medium gray */
--gray-600: #333333   /* Dark text */
--gray-700: #1A1A1A   /* Very dark */
--gray-800: #0D0D0D   /* Near black */
```

**Rules:**
- Never use colors outside this palette
- No gradients, shadows, or effects
- High contrast only (black on white, white on black)

## Typography

**Font Stack:**
```css
font-family: 'Courier New', 'Monaco', 'Menlo', 'Consolas', monospace;
```

**Scale:**
- Base: 14px
- Small: 11px-12px
- Medium: 13px-16px
- Large: 18px-24px

**Rules:**
- All text is uppercase for headings
- Letter-spacing: 1px-3px for emphasis
- Line-height: 1.5
- No font-weight variations beyond bold/normal

## Spacing System

**Base Unit:** 4px

**Scale:**
- 4px (1 unit) - Minimal spacing
- 8px (2 units) - Tight spacing
- 12px (3 units) - Small spacing
- 16px (4 units) - Standard spacing
- 24px (6 units) - Large spacing
- 32px (8 units) - Extra large spacing

**Rules:**
- Always use multiples of 4px
- Consistent padding/margins throughout

## Borders

**Standard:**
- 1px - Subtle dividers
- 2px - Standard borders
- 4px - Bold, brutalist borders

**Rules:**
- No border-radius (sharp corners only)
- Black borders only
- Geometric shapes only

## Components

### Buttons
```css
.button {
    padding: 12px 24px;
    border: 4px solid var(--black);
    background: var(--white);
    color: var(--black);
    font-family: monospace;
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 1px;
    cursor: pointer;
}

.button:hover {
    background: var(--black);
    color: var(--white);
}
```

### Cards/Boxes
```css
.card {
    border: 4px solid var(--black);
    padding: 24px 32px;
    background: var(--white);
}
```

### Input Fields
```css
.input {
    padding: 12px 16px;
    border: 4px solid var(--black);
    background: var(--white);
    color: var(--black);
    font-family: monospace;
    font-size: 13px;
}
```

### Code Blocks
```css
code {
    background: var(--black);
    color: var(--white);
    padding: 16px;
    border: 2px solid var(--black);
    font-family: monospace;
    font-size: 12px;
}
```

## Layout Principles

1. **Grid System**: Use flexbox, no complex grids
2. **Containers**: Max-width 1600px, centered
3. **Sidebars**: Fixed width (300px), sticky positioning
4. **No Shadows**: Flat design only
5. **No Gradients**: Solid colors only
6. **No Animations**: Instant state changes only

## Scrollbars

```css
::-webkit-scrollbar {
    width: 12px;
}

::-webkit-scrollbar-track {
    background: var(--white);
    border-left: 2px solid var(--black);
}

::-webkit-scrollbar-thumb {
    background: var(--black);
    border: 2px solid var(--white);
}
```

## Do's and Don'ts

### ✅ DO:
- Use monospace fonts everywhere
- Use 4px border increments
- Maintain high contrast
- Use uppercase for headings
- Keep spacing consistent
- Use geometric shapes only

### ❌ DON'T:
- Use colors outside the palette
- Add border-radius
- Use shadows or gradients
- Mix font families
- Use lowercase headings
- Add animations or transitions
- Use decorative elements

## Implementation

This design system is embedded in the HTML documentation page and should be extracted into a shared stylesheet for all frontend components.

**Location:** `src/main.rs` - `api_docs_html()` function

**Future:** Extract to `static/styles/design-system.css` for reuse across all pages.

