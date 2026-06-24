<!--
  Skill glyph symbol defs (skill-glyph-1 .. skill-glyph-15).

  Mounts a hidden inline SVG carrying <symbol> defs so the page can render
  glyphs with `<svg><use href="#skill-glyph-N" color="…" /></svg>`. Keep in
  sync with the copy embedded in `Board.svelte`'s <defs> — that copy stays
  because the match-board SVG already needs them at render time.
-->
<svg
  xmlns="http://www.w3.org/2000/svg"
  width="0"
  height="0"
  aria-hidden="true"
  style="position: absolute; width: 0; height: 0; overflow: hidden;"
>
  <defs>
    <symbol id="skill-glyph-1" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 4 20 L 18 6" />
        <path d="M 18 6 L 14 6 L 18 6 L 18 10" />
        <path d="M 5 19 L 8 19 L 5 19 L 5 16" />
      </g>
    </symbol>
    <symbol id="skill-glyph-2" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 18 4 L 18 14 A 6 6 0 0 1 6 14" />
        <path d="M 6 14 L 4 12" />
        <path d="M 6 14 L 4 16" />
      </g>
    </symbol>
    <symbol id="skill-glyph-3" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 6 5 L 18 5 L 17 14 L 12 19 L 7 14 Z" />
        <path d="M 12 5 L 9 11 L 13 13 L 11 19" />
      </g>
    </symbol>
    <symbol id="skill-glyph-4" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="16" cy="8" r="3" />
        <path d="M 4 18 L 8 14 L 14 18 L 10 22 Z" />
      </g>
    </symbol>
    <symbol id="skill-glyph-5" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="2.5" />
        <path d="M 12 4 L 12 7" />
        <path d="M 12 17 L 12 20" />
        <path d="M 4 12 L 7 12" />
        <path d="M 17 12 L 20 12" />
        <path d="M 6 6 L 8 8" />
        <path d="M 16 16 L 18 18" />
        <path d="M 6 18 L 8 16" />
        <path d="M 16 8 L 18 6" />
      </g>
    </symbol>
    <symbol id="skill-glyph-6" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 12 3 L 19 7 L 18 15 L 12 21 L 6 15 L 5 7 Z" />
      </g>
    </symbol>
    <symbol id="skill-glyph-7" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="8" />
        <path d="M 12 7 L 12 17" />
        <path d="M 7 12 L 17 12" />
      </g>
    </symbol>
    <symbol id="skill-glyph-8" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 5 8 L 19 8" />
        <path d="M 6 13 L 18 13" />
        <path d="M 8 18 L 16 18" />
      </g>
    </symbol>
    <symbol id="skill-glyph-9" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 4 12 L 14 12" />
        <path d="M 10 7 L 15 12 L 10 17" />
        <path d="M 14 7 L 19 12 L 14 17" />
      </g>
    </symbol>
    <symbol id="skill-glyph-10" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="6" cy="12" r="2.5" />
        <path d="M 11 12 L 21 12" />
        <path d="M 18 8 L 22 12 L 18 16" />
      </g>
    </symbol>
    <symbol id="skill-glyph-11" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="2.2" />
        <path d="M 10 5 L 12 3 L 14 5" />
        <path d="M 10 19 L 12 21 L 14 19" />
        <path d="M 5 10 L 3 12 L 5 14" />
        <path d="M 19 10 L 21 12 L 19 14" />
      </g>
    </symbol>
    <symbol id="skill-glyph-12" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 5 9 L 18 9" />
        <path d="M 15 5 L 19 9 L 15 13" />
        <path d="M 19 15 L 6 15" />
        <path d="M 9 11 L 5 15 L 9 19" />
      </g>
    </symbol>
    <symbol id="skill-glyph-13" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 6 6 L 6 11 A 5 5 0 0 0 16 11 L 16 18" />
        <path d="M 13 15 L 16 18 L 19 15" />
      </g>
    </symbol>
    <symbol id="skill-glyph-14" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="2" />
        <circle cx="12" cy="12" r="6" />
        <circle cx="12" cy="12" r="10" stroke-dasharray="2 3" />
      </g>
    </symbol>
    <symbol id="skill-glyph-15" viewBox="0 0 24 24">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M 13 3 L 6 14 L 11 14 L 9 21 L 18 9 L 13 9 Z" />
      </g>
    </symbol>
  </defs>
</svg>
