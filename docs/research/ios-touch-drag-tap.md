<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# RESEARCH REQUEST: iOS Safari touch input — tap vs drag disambiguation for a grid-based board game PWA

Context: I am building a single-file PWA (no framework, vanilla JS) that runs in iOS Safari as a board game prototype. The game has a 10×10 grid. Each cell can contain a game piece. The desired interaction model is:

- Short tap on a piece → open a detail modal (no movement)
- Drag gesture on a piece → move the piece to another cell (drag-and-drop)
The HTML5 Drag and Drop API does not work on iOS Safari. The current implementation using touchstart/touchmove/touchend with a pixel-distance threshold (8px) is unreliable — taps sometimes trigger drags and the modal sometimes doesn't open.

Questions:

1. What is the most reliable pattern for disambiguating a tap from a drag on iOS Safari using Touch Events or the Pointer Events API? Should I use a distance threshold, a time threshold (longpress), or a combination? What are the exact px/ms values used in production implementations?
2. Does iOS Safari support the Pointer Events API (pointerdown/pointermove/pointerup) reliably in 2024–2025, and is it preferable to Touch Events for this use case? What are the gotchas (passive listeners, touch-action CSS, preventDefault requirements)?
3. How do I reliably prevent the synthetic "ghost click" that iOS fires after touchend from accidentally triggering the modal click handler when the user intended a drag? What is the correct pattern to suppress it?
4. What CSS properties are required on draggable elements and their containers to prevent iOS Safari from interfering with custom touch handling (e.g. -webkit-user-select, touch-action, -webkit-touch-callout)?
5. Are there open-source board game PWAs or grid-based drag-and-drop libraries (e.g. Sortable.js, interact.js) that solve this exact problem and whose source or approach I can reference?

Please include:

- Concrete code patterns (not just philosophy)
- Known iOS Safari version-specific bugs or regressions relevant to this
- The touch-action CSS values that allow custom drag while still permitting page scroll outside the board
- Any relevant differences between Safari on iPad vs iPhone for pointer/touch handling

I now have comprehensive research across all five questions. Let me write the complete answer.

## iOS Safari Touch Disambiguation for a Grid PWA — Complete Reference

The core problem is that iOS Safari's touch model makes it impossible to commit to "tap" vs "drag" at `touchstart` time — you must defer that decision until `touchmove` or `touchend` while carefully managing side-effects (ghost clicks, passive listeners, default scroll) in the window between. Below is a production-grade guide to every dimension of this problem.

***

## 1. Tap vs. Drag Disambiguation Strategy

### Why a distance threshold alone fails

An 8 px distance threshold fails because iOS applies a hardware-level **jitter filter** that can already move the reported touch point 3–6 px during what the user experiences as a stationary tap . On older or sensitive digitizers (e.g., some iPad Pro models), this produces false drag detections constantly.

### The recommended combined approach

Use **both** a distance threshold **and** a time threshold together. Never decide at `touchstart`; decide lazily in `touchmove` or `touchend`:

```js
const DRAG_DISTANCE_THRESHOLD = 10;  // px — Euclidean, not Manhattan
const DRAG_TIME_THRESHOLD = 100;     // ms — elapsed since touchstart
const LONG_PRESS_MS = 300;           // ms — optional longpress-to-drag mode

let touchState = null;

piece.addEventListener('touchstart', (e) => {
  const t = e.changedTouches[^0];
  touchState = {
    id: t.identifier,
    startX: t.clientX,
    startY: t.clientY,
    startTime: Date.now(),
    isDrag: false,
    draggingStarted: false,
  };
  // Do NOT call e.preventDefault() here — you'll break scroll disambiguation
}, { passive: true });
```

In `touchmove`, you commit to drag only once **both** thresholds are exceeded:

```js
piece.addEventListener('touchmove', (e) => {
  if (!touchState) return;
  const t = [...e.changedTouches].find(c => c.identifier === touchState.id);
  if (!t) return;

  const dx = t.clientX - touchState.startX;
  const dy = t.clientY - touchState.startY;
  const dist = Math.hypot(dx, dy);
  const elapsed = Date.now() - touchState.startTime;

  if (!touchState.isDrag && dist > DRAG_DISTANCE_THRESHOLD && elapsed > DRAG_TIME_THRESHOLD) {
    touchState.isDrag = true;
    touchState.draggingStarted = true;
    e.preventDefault(); // safe here — non-passive, prevents scroll during drag
    startDrag(touchState, t);
  }

  if (touchState.isDrag) {
    e.preventDefault();
    updateDrag(t);
  }
}, { passive: false }); // MUST be non-passive to call preventDefault
```

In `touchend`, dispatch either tap or drop:

```js
piece.addEventListener('touchend', (e) => {
  if (!touchState) return;
  const t = [...e.changedTouches].find(c => c.identifier === touchState.id);
  if (!t) return;

  if (touchState.isDrag) {
    finalizeDrop(t);
  } else {
    openModal(); // tap — only fires if no drag was committed
  }
  touchState = null;
}, { passive: false });
```


### Production threshold values

Based on what major mobile frameworks (Sortable.js, interact.js, use-gesture) use internally :


| Parameter | Value | Rationale |
| :-- | :-- | :-- |
| Distance threshold | **10 px** Euclidean | Covers hardware jitter; SortableJS default `touchStartThreshold` |
| Time gate (minimum) | **100–150 ms** | Prevents accidental drag on very fast taps |
| Long-press-to-drag | **300 ms** hold | Optional; best for grid pieces that coexist with scroll |
| Ghost click window | **350 ms** post-touchend | The maximum iOS click-synthesis delay |

Using a **long-press** model (hold 300 ms to initiate drag) is particularly ergonomic for board games because pieces sit inside a scrollable container — it gives the browser time to decide whether the user is scrolling before you take over .

***

## 2. Pointer Events API — iOS Support Status and Gotchas

### Support status

The Pointer Events API has been supported in Safari since **Safari 13 (iOS 13, 2019)** . As of 2024–2025, it is fully available on all iOS/iPadOS versions your users will realistically have. It is the **preferred** approach because:

- Single event model for touch, Apple Pencil, and future stylus input
- `setPointerCapture` routes all future events to the captured element even when the finger moves off it — critical for drag across grid cells
- `pointercancel` fires reliably when the browser takes over a gesture (e.g., system scroll), allowing you to clean up drag state


### Critical gotchas

**Gotcha 1: `setPointerCapture` bug in iOS 13–15.4**
Calling `setPointerCapture` on an element that was *not* the original event target (i.e., a parent) was broken — it set the capture flag but didn't actually redirect events . This was mostly fixed in **iOS 15.5**. For older iOS targets, apply the polyfill:

```js
// Detect and polyfill setPointerCapture for iOS 13–15.4
if (navigator.userAgent.match(/Version\/1[^345]\.\d+(?:\.\d+)? Safari/)) {
  // See: https://stackoverflow.com/a/64018050
  // Polyfill patches Element.prototype.setPointerCapture
}
```

**Gotcha 2: `touch-action` must be set before `pointerdown`**
iOS Safari reads `touch-action` at gesture-start time. Setting it dynamically afterward — even with inline styles — is unreliable in iOS 15 . Set it in your initial CSS.

**Gotcha 3: `movementX`/`movementY` were undefined in Safari < 13**
As of Safari 13+, this is fixed, but always compute deltas manually from `clientX/Y` for safety .

**Gotcha 4: `pointerover`/`pointerenter` do not fire during touch drag on mobile**
You cannot use `pointerenter` to detect which cell the piece is hovering over. Instead, use `document.elementFromPoint(e.clientX, e.clientY)` inside `pointermove` to perform your own hit-testing :

```js
function getCellUnderPointer(e) {
  // Must temporarily hide the dragged clone so it doesn't block hit-testing
  dragClone.style.pointerEvents = 'none';
  const el = document.elementFromPoint(e.clientX, e.clientY);
  dragClone.style.pointerEvents = '';
  return el?.closest('.cell');
}
```

**Gotcha 5: iOS 17.4 touch event stoppage regression**
iOS 17.4.1 introduced a regression where `touchstart` events **stop firing entirely** after a random number of interactions, specifically when your handler triggers DOM repaints (e.g., canvas draws via `requestAnimationFrame`) . The workaround is to attach touch/pointer listeners to `document.body` rather than individual elements, and to debounce DOM updates away from the touch handler:

```js
// Attach to body/document rather than individual cells
document.body.addEventListener('touchstart', handler, { passive: false });

// Debounce any canvas/DOM repaint triggered by touch
function updateDrag(t) {
  requestAnimationFrame(() => renderDragState(t.clientX, t.clientY));
}
```

This regression was confirmed on iPad 6th gen, iPad 9th gen, iPad Pro M2, and iPhone 13 Pro under iOS 17.4.1 and 17.5 beta .

### Pointer Events implementation skeleton

```js
const DRAG_THRESHOLD = 10;
let dragState = null;

piece.addEventListener('pointerdown', (e) => {
  if (!e.isPrimary) return; // ignore secondary touch points
  e.preventDefault();       // prevent ghost click generation
  piece.setPointerCapture(e.pointerId); // routes all future pointer events here
  dragState = {
    pointerId: e.pointerId,
    startX: e.clientX,
    startY: e.clientY,
    startTime: Date.now(),
    isDrag: false,
  };
});

piece.addEventListener('pointermove', (e) => {
  if (!dragState || e.pointerId !== dragState.pointerId) return;
  const dist = Math.hypot(e.clientX - dragState.startX, e.clientY - dragState.startY);
  if (!dragState.isDrag && dist > DRAG_THRESHOLD && (Date.now() - dragState.startTime) > 100) {
    dragState.isDrag = true;
  }
  if (dragState.isDrag) {
    const targetCell = getCellUnderPointer(e);
    highlightCell(targetCell);
    moveDragClone(e.clientX, e.clientY);
  }
});

piece.addEventListener('pointerup', (e) => {
  if (!dragState) return;
  if (dragState.isDrag) {
    const cell = getCellUnderPointer(e);
    dropPieceOnCell(cell);
  } else {
    openModal(piece);
  }
  piece.releasePointerCapture(e.pointerId);
  dragState = null;
});

piece.addEventListener('pointercancel', (e) => {
  // Browser took over (scroll, home bar swipe, incoming call)
  cancelDrag();
  dragState = null;
});
```


***

## 3. Ghost Click Prevention

A "ghost click" is the synthetic `click` event iOS fires approximately **300–350 ms after `touchend`** to maintain backward compatibility with click-only sites . When `viewport` includes `width=device-width`, modern iOS eliminates the delay for standard taps , but during a *drag that ends without movement*, or when using `pointerdown`/`pointerup`, the ghost click can still fire and accidentally open your modal.

### Pattern A — `preventDefault` on `pointerdown` (simplest)

Calling `e.preventDefault()` in `pointerdown` suppresses the entire subsequent synthetic mouse/click event chain :

```js
piece.addEventListener('pointerdown', (e) => {
  e.preventDefault(); // kills ghost click chain for this gesture
  piece.setPointerCapture(e.pointerId);
  // ...
});
```

**Caveat:** This also suppresses focus/keyboard for `<input>` elements inside cells, but for non-input game pieces it's fine.

### Pattern B — Timestamp flag (for touch events fallback)

If you need to keep the click event for accessibility or keyboard fallback, suppress it with a timestamp guard:

```js
let lastTouchEndTime = 0;

piece.addEventListener('touchend', (e) => {
  lastTouchEndTime = Date.now();
  if (touchState && !touchState.isDrag) {
    openModal(piece);
  }
  // Do NOT call preventDefault unconditionally — kills momentum scroll
});

piece.addEventListener('click', (e) => {
  // Ghost clicks arrive within ~400ms of touchend
  if (Date.now() - lastTouchEndTime < 400) {
    e.stopPropagation();
    return; // swallow ghost
  }
  openModal(piece); // real mouse click (desktop fallback)
});
```


### Pattern C — Coordinate filter (most robust)

Track the last touch-end coordinates and reject any `click` that fires within 400 ms at those same coordinates :

```js
const suppressedClicks = new Set(); // stores "x,y" keys

piece.addEventListener('touchend', (e) => {
  const t = e.changedTouches[^0];
  const key = `${Math.round(t.clientX)},${Math.round(t.clientY)}`;
  suppressedClicks.add(key);
  setTimeout(() => suppressedClicks.delete(key), 400);
});

piece.addEventListener('click', (e) => {
  const key = `${Math.round(e.clientX)},${Math.round(e.clientY)}`;
  if (suppressedClicks.has(key)) { e.stopPropagation(); return; }
  openModal(piece);
});
```


***

## 4. Required CSS Properties

### On the game board container

```css
.game-board {
  /* Disable ALL browser touch handling within the board.
     Required for custom drag; prevents scroll hijacking the drag gesture. */
  touch-action: none;

  /* Prevent iOS text-selection popup during long press / drag */
  -webkit-user-select: none;
  user-select: none;

  /* Prevent the iOS magnifier loupe and link-preview menu */
  -webkit-touch-callout: none;

  /* Kills the grey tap flash on iOS */
  -webkit-tap-highlight-color: transparent;
}
```


### On individual pieces (draggable elements)

```css
.piece {
  touch-action: none;          /* must be set before pointerdown */
  -webkit-user-select: none;
  user-select: none;
  -webkit-touch-callout: none;
  cursor: pointer;             /* required for iOS Safari click events to fire */
  will-change: transform;      /* GPU layer — avoids jank during drag */
}
```


### Coexisting scroll *outside* the board

This is the key pattern for allowing the page to scroll above/below the board while disabling it inside :

```css
/* Page wrapper — allow normal scroll everywhere */
body {
  touch-action: auto;
}

/* Board area — disable all browser gestures; JS handles everything */
.game-board {
  touch-action: none;
}

/* Alternatively, if you want pinch-zoom on board but custom drag: */
.game-board {
  touch-action: pinch-zoom;
  /* then prevent-default in pointermove once drag is committed */
}
```

```
Do **not** set `touch-action: none` on `<html>` or `<body>` — this kills page scroll globally. Apply it only to the board element and its piece children .
```


### The `manipulation` shortcut (simpler alternative)

`touch-action: manipulation` disables double-tap-to-zoom and thus removes the 300 ms click delay, while still allowing single-finger scroll . Useful if your pieces are inside a scrollable container and you don't need full drag:

```css
.piece {
  touch-action: manipulation; /* no double-tap zoom, but scroll still works */
}
```


***

## 5. Libraries and Reference Implementations

### interact.js

**Best fit for your use case.** interact.js uses the Pointer Events API with Touch Events fallback, handles `setPointerCapture`, and exposes `dragstart`/`dragmove`/`dragend` events with built-in iOS quirk handling . Crucially, it **does not move elements itself** — you apply transforms from your event handlers, which suits a grid-snapping model:

```js
interact('.piece').draggable({
  listeners: {
    start(event) { /* set isDragging flag, prevent modal open */ },
    move(event) {
      const x = (parseFloat(event.target.getAttribute('data-x')) || 0) + event.dx;
      const y = (parseFloat(event.target.getAttribute('data-y')) || 0) + event.dy;
      event.target.style.transform = `translate(${x}px, ${y}px)`;
      event.target.setAttribute('data-x', x);
      event.target.setAttribute('data-y', y);
    },
    end(event) {
      const cell = getCellUnderPointer(event);
      snapToCell(event.target, cell);
    }
  }
}).on('tap', (event) => openModal(event.target));
// interact.js fires 'tap' only when no drag occurred
```

Known iOS quirk in older interact.js versions: iOS would fire 5 simultaneous pointer events on some iPads, registering the sequence as a "gesture" and aborting the drag . This is fixed in interact.js ≥1.3.

### SortableJS (`forceFallback: true`)

Sortable.js defaults to HTML5 Drag and Drop but supports a **fallback mode** that uses Touch/Pointer Events, which is required for iOS :

```js
Sortable.create(boardElement, {
  forceFallback: true,         // required for iOS Safari
  fallbackOnBody: true,
  delay: 200,                  // ms hold before drag — prevents tap misfire
  delayOnTouchOnly: true,      // only apply delay on touch
  touchStartThreshold: 5,      // px before drag starts
  animation: 150,
});
```

Sortable also needs `group` defined on iOS; without it, drag fails silently on many iOS versions .

### Red Blob Games draggable demo

Amit Patel's [redblobgames.com draggable article](https://www.redblobgames.com/x/2251-draggable/) (updated 2024) is an excellent annotated reference for Pointer Events patterns including `setPointerCapture`, browser compatibility notes, and the `document.elementFromPoint` hit-testing pattern . Highly recommended as study material.

***

## iPad vs. iPhone Differences

| Behavior | iPhone | iPad |
| :-- | :-- | :-- |
| **Pointer offset bug (iPadOS 17.3)** | Rare | Common — events fire ~100px to the left of actual pointer after scroll |
| **Apple Pencil** | N/A | Sends `PointerEvent` with `pointerType: "pen"` and pressure data; `pointermove` sampling rate is *lower* than Touch Events |
| **Multi-touch** | Identical API | Can trigger false "gesture" detection with 5 simultaneous pointer events in older interact.js |
| **PWA standalone home bar** | Bottom home bar can generate a `pointercancel` mid-drag if user accidentally swipes up | Same, more frequent on iPad with stage-manager |
| **Double-tap zoom** | Disabled when `width=device-width` viewport set | Same, but keyboard-attached iPads may have different behavior |

For the pointer offset bug (iPadOS 17.3.x), there is no JS fix — it was a WebKit rendering bug fixed in 17.4+. Users on 17.3 must update .

***

## Passive Listener Checklist

From iOS 11.3 onward, `touchstart` and `touchmove` on `document` are **passive by default**, meaning `preventDefault()` silently does nothing unless you opt out :

```js
// ✗ WRONG — passive by default, preventDefault() is ignored
element.addEventListener('touchmove', handler);

// ✓ CORRECT — explicit opt-out
element.addEventListener('touchmove', handler, { passive: false });

// ✓ OK — you're not calling preventDefault(), so passive is fine
element.addEventListener('touchstart', trackStart, { passive: true });
```

The rule of thumb: attach `{ passive: true }` to `touchstart` (you never need `preventDefault` there if using the deferred disambiguation pattern), and `{ passive: false }` only to `touchmove` and `touchend` where you actively call `preventDefault()` to block scroll during confirmed drags .

***

## Minimal Standalone PWA `<head>` Configuration

These meta tags are required to eliminate the 300 ms tap delay and ensure correct viewport behavior :

```html
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="apple-mobile-web-app-capable" content="yes">
<meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
```

The `width=device-width` viewport tag removes the 350 ms tap delay in WebKit without needing any library . The `apple-mobile-web-app-capable` puts the PWA in standalone mode where the iOS bottom browser toolbar (which can intercept low swipes) is absent.
<span style="display:none">[^1][^10][^100][^101][^102][^103][^104][^105][^106][^107][^108][^109][^11][^110][^111][^112][^113][^114][^115][^116][^117][^118][^119][^12][^120][^121][^122][^123][^124][^125][^126][^127][^128][^129][^13][^130][^14][^15][^16][^17][^18][^19][^2][^20][^21][^22][^23][^24][^25][^26][^27][^28][^29][^3][^30][^31][^32][^33][^34][^35][^36][^37][^38][^39][^4][^40][^41][^42][^43][^44][^45][^46][^47][^48][^49][^5][^50][^51][^52][^53][^54][^55][^56][^57][^58][^59][^6][^60][^61][^62][^63][^64][^65][^66][^67][^68][^69][^7][^70][^71][^72][^73][^74][^75][^76][^77][^78][^79][^8][^80][^81][^82][^83][^84][^85][^86][^87][^88][^89][^9][^90][^91][^92][^93][^94][^95][^96][^97][^98][^99]</span>

<div align="center">⁂</div>

[^1]: https://discourse.threejs.org/t/iphone-safari-touch-events-issue/54328

[^2]: https://dev.to/pfacklam/consistent-access-to-browser-events-with-pointer-events-api-5gh4

[^3]: https://www.cnblogs.com/lifeisshort/p/4895632.html

[^4]: https://stackoverflow.com/questions/78380740/ios-safari-touch-events-stop-firing-with-17-4-1

[^5]: https://stackoverflow.com/questions/56463451/workaround-for-safari-ios-pointer-events-not-supported

[^6]: https://github.com/aamnah/notes/blob/main/javascript/Events-clicks-taps-mobile-browsers-ios.md

[^7]: https://stackoverflow.com/questions/2890898/preventing-mouse-emulation-events-i-e-click-from-touch-events-in-mobile-safar

[^8]: https://github.com/facebook/react/issues/12901

[^9]: https://stackoverflow.com/questions/13219017/ghostclicks-in-mobile-apps

[^10]: https://developer.apple.com/library/archive/documentation/AppleApplications/Reference/SafariWebContent/HandlingEvents/HandlingEvents.html

[^11]: https://stackoverflow.com/questions/69450411/setting-pointer-events-dynamically-on-ios-15-safari-is-unreliable-and-unpredicta

[^12]: https://stackoverflow.com/questions/20359872/prevent-click-event-after-handling-jquery-mobile-tap-event-on-ios

[^13]: https://www.reddit.com/r/webdev/comments/xyzma9/safari_16_on_ios_not_firing_some_touch_events_on/

[^14]: https://www.reddit.com/r/iOSProgramming/comments/bymsuc/does_safari_13_fully_support_pointer_events_dont/

[^15]: https://gist.github.com/terrymun/967157a6a328ff17e873b425103dd733

[^16]: https://stackoverflow.com/questions/51600527/ios-11-4-safari-not-respecting-touch-action-manipulation

[^17]: https://stackoverflow.com/questions/33033089/interact-js-1-2-4-drag-and-gesture-action-confusion

[^18]: https://forum.babylonjs.com/t/prevent-unwanted-inputs-in-ios-safari-hold-click-selects-canvas/21409

[^19]: https://github.com/pmndrs/use-gesture/issues/441

[^20]: https://interactjs.io

[^21]: https://css-tricks.com/almanac/properties/t/touch-action/

[^22]: https://github.com/pmndrs/use-gesture/issues/349

[^23]: https://hacks.mozilla.org/2014/11/interact-js-for-drag-and-drop-resizing-and-multi-touch-gestures/

[^24]: https://github.com/pmndrs/use-gesture/issues/486

[^25]: https://github.com/mastodon/mastodon/issues/29624

[^26]: https://github.com/taye/interact.js/issues/595

[^27]: https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/touch-action

[^28]: https://github.com/openlayers/openlayers/issues/14703

[^29]: https://github.com/SortableJS/Sortable/issues/1319

[^30]: https://stackoverflow.com/questions/1282504/double-tap-or-two-single-taps

[^31]: https://support.apple.com/en-eg/guide/logicremote-logicpro-iphone/chs71110e4a6/ios

[^32]: https://github.com/kamranahmedse/driver.js/issues/462

[^33]: https://patents.google.com/patent/US10860199B2/en

[^34]: https://github.com/shinsenter/defer.js/discussions/122

[^35]: https://www.sitepoint.com/5-ways-prevent-300ms-click-delay-mobile-devices/

[^36]: https://dl.acm.org/doi/10.1145/3706598.3713376

[^37]: https://stackoverflow.com/questions/63487102/touch-events-are-not-handled-on-iphone-safari-when-page-scrolled

[^38]: https://www.medien.ifi.lmu.de/lehre/ws1112/mmi2/slides/MMI2-04-InputOutput.pdf

[^39]: https://developer.apple.com/forums/topics/safari-and-web-topic/safari-and-web-topic-general?sortBy=lastUpdated

[^40]: https://webkit.org/blog/5610/more-responsive-tapping-on-ios/

[^41]: https://stackoverflow.com/questions/5348092/prevent-default-press-but-not-default-drag-in-ios-mobilesafari

[^42]: https://www.npmjs.com/package/sortablejs

[^43]: https://community.latenode.com/t/handling-click-events-inside-touchmove-containers-on-mobile-safari/32705

[^44]: https://www.cssscript.com/lightweight-js-sorting-library-with-native-html5-drag-and-drop-sortable/

[^45]: https://www.drupal.org/project/entity_browser/issues/3364018

[^46]: https://caniuse.com/css-touch-action

[^47]: https://github.com/Splidejs/splide/discussions/625

[^48]: https://github.com/SortableJS/Sortable/issues/997

[^49]: https://sortablejs.github.io/Sortable/

[^50]: https://developer.mozilla.org/de/docs/Web/CSS/Reference/Properties/touch-action

[^51]: https://medium.com/@_ric/why-you-should-be-using-pointer-events-5b1e68171bac

[^52]: https://github.com/w3c/pointerevents/issues/346

[^53]: https://forum.babylonjs.com/t/safari-pointerevents-bug/2712

[^54]: https://stackoverflow.com/questions/38671519/why-would-my-touchstart-and-touchend-events-not-fire-on-safari-mobile

[^55]: https://www.redblobgames.com/x/2251-draggable/

[^56]: https://meta.discourse.org/t/ios-26-bugs-with-fixed-position-elements-in-discourse/382831?page=2

[^57]: https://developer.apple.com/forums/thread/776468

[^58]: https://community.bear.app/t/cursor-not-showing-ios-ipados-17-bug-just-an-fyi/11057

[^59]: https://aamnah.com/notes/javascript/events-clicks-taps-mobile-browsers-ios/

[^60]: https://patrickhlauke.github.io/touch/tests/results/

[^61]: https://stackoverflow.com/questions/8809706/ios-5-safari-bug-with-html-touch-events-on-positionfixed-div

[^62]: https://dev.to/nishinoshake/smooth-drag-interactions-with-pointer-events-5e2j

[^63]: https://www.uriports.com/blog/easy-fix-for-unable-to-preventdefault-inside-passive-event-listener/

[^64]: https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events/Multi-touch_interaction

[^65]: https://github.com/SortableJS/Sortable/issues/2426

[^66]: https://www.redblobgames.com/x/2305-passive-events/

[^67]: https://www.w3.org/TR/pointerevents/

[^68]: https://blog.gitcode.com/60885ded4c527cb46eaca513a67826d0.html

[^69]: https://github.com/ohcnetwork/care_fe/issues/12609

[^70]: https://github.com/mattdesl/simple-input-events

[^71]: https://stackoverflow.com/questions/61199109/sortablejs-mobile-implementation-of-drag-n-drop-via-touch

[^72]: https://github.com/necolas/react-native-web/issues/1505

[^73]: https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events

[^74]: https://stackoverflow.com/questions/49500339/cant-prevent-touchmove-from-scrolling-window-on-ios

[^75]: https://forum.babylonjs.com/t/safari-12-on-ios-doesnt-respond-to-drag-camera-events-desktop-works/25540

[^76]: https://medium.com/codex/drag-n-drop-with-vanilla-javascript-75f9c396ecd

[^77]: https://stackoverflow.com/questions/8839333/multitouch-touchevents-not-triggered-as-they-should-on-safari-mobile

[^78]: https://blog.r0b.io/post/creating-drag-interactions-with-set-pointer-capture-in-java-script/

[^79]: https://stackoverflow.com/questions/50541215/grid-game-board

[^80]: https://www.youtube.com/watch?v=sNn_Gxph3TY

[^81]: https://stackoverflow.com/questions/31985051/safari-not-firing-touch-events

[^82]: https://support.touch-base.com/Documentation/50584/Browser-TUIO-interface

[^83]: https://tahazsh.com/blog/seamless-ui-with-js-drag-to-reorder-example

[^84]: https://developer.mozilla.org/de/docs/Web/API/Element/setPointerCapture

[^85]: https://www.youtube.com/watch?v=FIyaIewZQsI

[^86]: https://dev.to/mpuckett/the-holy-grail-web-app-shell-with-header-and-footer-for-iphone-549j

[^87]: https://stackoverflow.com/questions/76428827/requestanimationframe-not-being-triggered-in-ios

[^88]: https://github.com/julianshapiro/velocity/issues/261

[^89]: https://medium.com/@yev-/how-to-prevent-scroll-touch-move-on-mobile-web-parent-elements-while-allowing-it-on-children-f7acb793c621

[^90]: https://stackoverflow.com/questions/64017560/detect-ios-pointer-capture-bug-for-polyfilling

[^91]: https://www.youtube.com/watch?v=4e4hX8vNvlo

[^92]: https://frontend-practical.dev/en/css/touch-action/

[^93]: https://developer.mozilla.org/en-US/docs/Web/API/Element/setPointerCapture

[^94]: https://agenxus.com/blog/wordpress-inp-performance-crisis

[^95]: https://github.com/vercel/next.js/issues/91908

[^96]: https://forum.keyboardmaestro.com/t/how-do-you-configure-the-long-press-time/37042

[^97]: https://www.youtube.com/watch?v=M60yN7_EvDY

[^98]: https://medium.com/@kristiantolleshaugmrch/fixing-the-double-tap-issue-in-ios-safari-with-javascript-4e72a18a1feb

[^99]: https://shop.hioki.eu/media/39/e6/7b/1736415340/MR6000A966-08_Manual_EN.pdf

[^100]: https://forum.playcanvas.com/t/does-not-work-touch-event-with-ipad-pro-ios-15-6-1/29031

[^101]: https://www.appypie.com/blog/mobile-app-animation-guide

[^102]: https://interactjs.io/docs/

[^103]: https://stackoverflow.com/questions/57334793/touch-click-and-input-event-listeners-not-firing-ios

[^104]: https://community.weweb.io/t/be-careful-with-new-grid-element-display-functionality-all-apple-mobile-devices-safari-on-mac-arent-supported/9481

[^105]: https://stackoverflow.com/questions/4669823/gotchas-bugs-in-development-for-webkit-on-ios-or-android

[^106]: https://www.telerik.com/forums/grid-not-displaying-on-ios-devices

[^107]: https://github.com/ionic-team/ionic-framework/issues/19299

[^108]: https://interactjs.io/docs/migrating/

[^109]: https://forum.bubble.io/t/drag-drop-group-not-working-on-mobile-phone/110029

[^110]: https://www.telerik.com/forums/web-ui-grid-display-problem-in-ipad-safari-(ios-7-)

[^111]: https://blog.mobiscroll.com/working-with-touch-events/

[^112]: https://sheet.shiar.nl/browser

[^113]: https://mobiforge.com/design-development/touch-friendly-drag-and-drop

[^114]: https://web.dev/articles/mobile-touch

[^115]: https://developer.apple.com/documentation/uikit/uiview/hittest(_:with:)

[^116]: https://stackoverflow.com/questions/5159366/cant-handle-both-click-and-touch-events-simultaneously

[^117]: https://github.com/clauderic/react-sortable-hoc/issues/253

[^118]: https://stackoverflow.com/questions/73481817/pointer-event-none-not-work-on-ios-safari

[^119]: https://demos.jquerymobile.com/1.2.0/docs/api/events.html

[^120]: https://stackoverflow.com/questions/74875307/javascript-touch-event-not-fired-when-clicking-repeatedly-in-standalone-pwa-on-i

[^121]: https://stackoverflow.com/questions/55120331/white-panel-arrives-on-double-tap-from-bottom-in-pwa-in-standalone-mode

[^122]: https://developer.apple.com/library/archive/documentation/AppleApplications/Reference/SafariWebContent/ConfiguringWebApplications/ConfiguringWebApplications.html

[^123]: https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Manifest/Reference/display

[^124]: https://www.denis.es/blog/how-pwa-behaves-on-different-platforms-ios-android-windows-and-browsers/

[^125]: https://www.mediaevent.de/javascript/progressive-web-app.html

[^126]: https://gist.github.com/fozzedout/5e77925381991a9570151550992baf14

[^127]: https://web.dev/learn/pwa/app-design

[^128]: https://firt.dev/notes/pwa-ios/

[^129]: https://xenforo.com/community/threads/allow-ios-fullscreen-pwa-by-adding-option-for-display-standalone-in-web-app-manifest.185435/

[^130]: https://www.magicbell.com/blog/pwa-ios-limitations-safari-support-complete-guide

