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
--green-primary: #00D977      /* Darker, more opaque green - primary text */
--green-secondary: #00B366    /* Even darker for secondary text */
--green-accent: #00FF88       /* Brighter for accents only */
--green-glow: rgba(0, 217, 119, 0.5)  /* Glow effect for text-shadow */
```

**Rules:**
- Never use colors outside this palette
- Use darker greens (#00D977) instead of bright Matrix green
- Add subtle glow effects with text-shadow
- High contrast for readability (WCAG AA+ compliant)
- Dark backgrounds with darker green accents

## Typography

**Font Stack:**
```css
font-family: 'IBM Plex Mono', 'Courier New', 'Monaco', 'Menlo', 'Consolas', monospace;
```

**Font Loading:**
```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600;700&display=swap" rel="stylesheet">
```

**Weights Available:**
- 400 (Regular)
- 500 (Medium)
- 600 (Semi-bold)
- 700 (Bold)

**Scale:**
- Base: 15px (improved from 14px for readability)
- Small: 11px-12px
- Medium: 13px-16px
- Large: 18px-28px

**Readability Rules:**
- Line-height: 1.7-1.8 for body text (improved from 1.5)
- Letter-spacing: 0.3px for body, 1px-3px for headings
- All text is uppercase for headings
- Font-weight: 400 (regular) for body, 600-700 for headings
- Max-width: 800px for descriptions (optimal reading width)
- Text-shadow glow effects for emphasis

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
    border: 4px solid var(--green-primary);
    background: var(--black);
    color: var(--green-primary);
    font-family: 'IBM Plex Mono', monospace;
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 1px;
    cursor: pointer;
}

.button:hover {
    background: var(--green-primary);
    color: var(--black);
}
```

### Cards/Boxes
```css
.card {
    border: 4px solid var(--green-primary);
    padding: 24px 32px;
    background: var(--black);
    color: var(--green-primary);
}
```

### Input Fields
```css
.input {
    padding: 12px 16px;
    border: 4px solid var(--green-primary);
    background: var(--dark);
    color: var(--green-primary);
    font-family: 'IBM Plex Mono', monospace;
    font-size: 13px;
}
```

### Code Blocks
```css
code {
    background: var(--black);
    color: var(--green-primary);
    padding: 16px;
    border: 2px solid var(--green-primary);
    font-family: 'IBM Plex Mono', monospace;
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

