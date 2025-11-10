# BRUTALIST DARK MODE DESIGN SYSTEM

## Philosophy

**Minimalist | Monospace | Geometric | Stark | Matrix**

A brutalist dark mode design system built on the principles of:
- **Brutalism**: Raw, bold, geometric forms with no decoration
- **Dark Mode**: Black backgrounds with Matrix green accents
- **Monospace**: Courier New/Monaco/Menlo fonts throughout
- **Minimalism**: Clean, functional, no unnecessary elements
- **Hacker Aesthetic**: Matrix green on black terminal-style

## Color Palette

```css
--black: #000000              /* Primary background */
--dark: #0A0A0A               /* Dark backgrounds */
--dark-gray: #1A1A1A          /* Medium dark */
--medium-gray: #2A2A2A        /* Lighter dark */
--light-gray: #3A3A3A         /* Light dark */
--white: #FFFFFF              /* Rarely used */
--matrix-green: #00FF41        /* Primary accent - Matrix green */
--matrix-green-dark: #00CC33   /* Darker green for secondary text */
--matrix-green-bright: #00FF88 /* Bright green for highlights */
```

**Rules:**
- Never use colors outside this palette
- No gradients, shadows, or effects
- High contrast only (Matrix green on black)
- Dark backgrounds with green accents

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
    border: 4px solid var(--matrix-green);
    background: var(--black);
    color: var(--matrix-green);
    font-family: monospace;
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 1px;
    cursor: pointer;
}

.button:hover {
    background: var(--matrix-green);
    color: var(--black);
}
```

### Cards/Boxes
```css
.card {
    border: 4px solid var(--matrix-green);
    padding: 24px 32px;
    background: var(--black);
    color: var(--matrix-green);
}
```

### Input Fields
```css
.input {
    padding: 12px 16px;
    border: 4px solid var(--matrix-green);
    background: var(--dark);
    color: var(--matrix-green);
    font-family: monospace;
    font-size: 13px;
}
```

### Code Blocks
```css
code {
    background: var(--black);
    color: var(--matrix-green);
    padding: 16px;
    border: 2px solid var(--matrix-green);
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
- Maintain high contrast (Matrix green on black)
- Use uppercase for headings
- Keep spacing consistent
- Use geometric shapes only
- Use dark backgrounds with green accents
- Keep the hacker/terminal aesthetic

### ❌ DON'T:
- Use colors outside the palette (especially no bright colors except Matrix green)
- Add border-radius (sharp corners only)
- Use shadows or gradients
- Mix font families
- Use lowercase headings
- Add animations or transitions
- Use decorative elements
- Use white backgrounds (too bright/shocking)
- Use colors other than black, dark grays, and Matrix green

## Implementation

This design system is embedded in the HTML documentation page and should be extracted into a shared stylesheet for all frontend components.

**Location:** `src/main.rs` - `api_docs_html()` function

**Future:** Extract to `static/styles/design-system.css` for reuse across all pages.

