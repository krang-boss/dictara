import { useEffect, useLayoutEffect, useRef } from 'react'
import type { PopupStyle } from '@/bindings'

/** What the orb is doing. Drives how it moves. */
export type OrbState = 'idle' | 'recording' | 'processing' | 'dispersing'

/**
 * The recording orb.
 *
 * A port of the `cloud` theme from orb-ui (MIT, © 2026 Alexander Chen —
 * https://github.com/alexanderqchen/orb-ui, `src/themes/cloud/CloudTheme.tsx`),
 * adapted to this app's `PopupStyle` and recording states.
 *
 * What changed from the original:
 *
 * - The four hard-coded periwinkle/milk colours became palette uniforms, so
 *   the orb can be recoloured from the laboratory.
 * - Speed, turbulence, softness, saturation and reactivity are wired to the
 *   laboratory sliders.
 * - The provider states (`listening`/`speaking`/`connecting`) map onto ours
 *   (`recording`/`processing`/`dispersing`).
 * - The finish is a plain fade-and-settle rather than a dispersal.
 *
 * Everything animated runs inside one rAF loop writing to `style` directly, so
 * a live style change never re-renders the React tree.
 */

const useIsomorphicLayoutEffect = typeof window !== 'undefined' ? useLayoutEffect : useEffect

interface OrbProps {
  style: PopupStyle
  state?: OrbState
  /** Live microphone level, 0..1. */
  level?: number
  /** Box size in CSS pixels; the cloud itself is a fraction of it. */
  size?: number
  paused?: boolean
  className?: string
}

interface CloudRenderer {
  draw(time: number, activity: number, style: PopupStyle): void
  destroy(): void
}

const VERTEX_SHADER = `
attribute vec2 a_position;

void main() {
  gl_Position = vec4(a_position, 0.0, 1.0);
}
`

const FRAGMENT_SHADER = `
precision highp float;

uniform vec2 u_resolution;
uniform float u_time;
uniform float u_activity;

uniform vec3 u_deep;    // densest sky, high in the sphere
uniform vec3 u_upper;   // the upper field
uniform vec3 u_lower;   // below the horizon
uniform vec3 u_milk;    // the lit cloud bank itself
uniform float u_warp;       // turbulence, 0..1
uniform float u_edge;       // softness, 0..1
uniform float u_saturation; // 0..1

float hash(vec2 p) {
  p = fract(p * vec2(123.34, 456.21));
  p += dot(p, p + 45.32);
  return fract(p.x * p.y);
}

float noise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  vec2 u = f * f * (3.0 - 2.0 * f);

  return mix(
    mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x),
    mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x),
    u.y
  );
}

float fbm(vec2 p) {
  float value = 0.0;
  float amplitude = 0.52;
  mat2 rotation = mat2(0.80, 0.60, -0.60, 0.80);

  for (int octave = 0; octave < 5; octave++) {
    value += amplitude * noise(p);
    p = rotation * p * 1.92 + vec2(9.7, 4.3);
    amplitude *= 0.5;
  }

  return value;
}

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  vec2 centered = uv - 0.5;
  float radius = length(centered);

  // Softness widens the falloff band instead of moving the rim, so the disc
  // keeps its size as it goes from a crisp bead to a diffuse puff.
  float feather = mix(0.012, 0.17, u_edge);
  float edge = 1.0 - smoothstep(0.5 - feather, 0.5, radius);

  if (edge <= 0.0) discard;

  vec2 p = centered * 2.0;
  float t = u_time;

  vec2 warp = vec2(
    fbm(p * 1.02 + vec2(t * 0.34, -t * 0.24)),
    fbm(p * 1.08 + vec2(-t * 0.27, t * 0.32) + vec2(6.7, 2.9))
  );
  vec2 curl = vec2(
    sin(p.y * 2.4 + t * 0.68 + warp.y * 3.2),
    cos(p.x * 2.1 - t * 0.61 + warp.x * 3.0)
  );
  float warpAmount = mix(0.72, 1.68, u_warp);
  vec2 warped =
    p +
    (warp - 0.5) * (warpAmount + u_activity * 0.38) +
    curl * (0.035 + u_activity * 0.07);
  float broad = fbm(warped * 0.92 + vec2(t * 0.14, -t * 0.18));
  float folded = fbm(warped * 1.66 + vec2(-t * 0.23, t * 0.19) + 5.2);
  float field = mix(broad, folded, 0.3 + u_activity * 0.14);

  float horizon =
    0.46 +
    0.08 * sin((uv.x + warp.x * 0.2) * 5.4 + t * 0.42) +
    0.16 * (broad - 0.5);
  float upper = smoothstep(horizon - 0.12, horizon + 0.08, uv.y);
  float band = exp(-pow((uv.y - horizon) * (5.2 + u_activity * 0.8), 2.0));
  float cloud = smoothstep(0.24, 0.79, field);

  vec3 color = mix(u_lower, u_upper, upper);
  float upperDepth = upper * (0.14 + smoothstep(0.42, 0.78, folded) * 0.5);
  color = mix(color, u_deep, upperDepth);

  float milkAmount = clamp(band * (0.42 + cloud * 0.62), 0.0, 0.88);
  color = mix(color, u_milk, milkAmount);

  float lowerMist = (1.0 - upper) * smoothstep(0.58, 0.9, broad) * 0.18;
  color = mix(color, u_milk, lowerMist);

  // Pull towards or away from the greyscale of the same brightness, so a low
  // saturation reads as overcast rather than as a darker sky.
  float luma = dot(color, vec3(0.299, 0.587, 0.114));
  color = clamp(mix(vec3(luma), color, 0.3 + u_saturation * 1.1), 0.0, 1.0);

  float grain = (noise(gl_FragCoord.xy * 0.64) - 0.5) / 255.0;
  color += grain;

  // The canvas is premultiplied, so the disc's own alpha has to be folded in.
  gl_FragColor = vec4(color * edge, edge);
}
`

/** The cloud's diameter as a fraction of the box, leaving room to overshoot. */
const NEUTRAL_DIAMETER = 0.66
const LISTEN_SHRINK = 0.204
const SPEAK_GROW = 0.2145
const DOT_SCALE = 0.063
const ENTRANCE_OVERSHOOT = 1.178
const DOT_HOLD_MS = 180
const GROW_MS = 300
const SETTLE_MS = 1350
const SURFACE_FADE_START_MS = DOT_HOLD_MS + GROW_MS * 0.22
const SURFACE_FADE_END_MS = DOT_HOLD_MS + GROW_MS * 0.8
const DOT_FADE_START_MS = DOT_HOLD_MS + GROW_MS * 0.58
const DOT_FADE_END_MS = DOT_HOLD_MS + GROW_MS

/** Palette slots, in the order the shader wants them. */
const FALLBACK_PALETTE = ['#5c63fb', '#7a8ffb', '#b8c7f9', '#e3ebfe']

function clamp(value: number, min = 0, max = 1) {
  return Math.min(max, Math.max(min, value))
}

function damp(current: number, target: number, rate: number, deltaSeconds: number) {
  return current + (target - current) * (1 - Math.exp(-rate * deltaSeconds))
}

function easeOutCubic(value: number) {
  return 1 - Math.pow(1 - value, 3)
}

function mix(from: number, to: number, progress: number) {
  return from + (to - from) * progress
}

function smoothstepRange(value: number, start: number, end: number) {
  const progress = clamp((value - start) / (end - start))
  return progress * progress * (3 - 2 * progress)
}

function hexToRgb(hex: string): [number, number, number] {
  const clean = hex.replace('#', '')
  const value = parseInt(clean.slice(0, 6), 16)
  if (Number.isNaN(value)) return [1, 1, 1]

  return [((value >> 16) & 255) / 255, ((value >> 8) & 255) / 255, (value & 255) / 255]
}

/**
 * Spread whatever the palette holds across the shader's four slots.
 *
 * The cloud needs a ramp from its darkest sky to its lit bank, so a shorter
 * palette is stretched rather than padded — two colours still produce a
 * readable sky, they just read flatter.
 */
function cloudColors(palette: string[]): [number, number, number][] {
  const stops = palette.filter(Boolean)
  const source = stops.length >= 2 ? stops : FALLBACK_PALETTE

  return [0, 1, 2, 3].map((slot) => {
    const position = (slot / 3) * (source.length - 1)
    const low = Math.floor(position)
    const high = Math.min(source.length - 1, low + 1)
    const blend = position - low

    const a = hexToRgb(source[low])
    const b = hexToRgb(source[high])
    return [mix(a[0], b[0], blend), mix(a[1], b[1], blend), mix(a[2], b[2], blend)]
  }) as [number, number, number][]
}

function compileShader(
  gl: WebGLRenderingContext,
  type: number,
  source: string
): WebGLShader | undefined {
  const shader = gl.createShader(type)
  if (!shader) return undefined

  gl.shaderSource(shader, source)
  gl.compileShader(shader)

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    console.error('Cloud shader compile failed:', gl.getShaderInfoLog(shader))
    gl.deleteShader(shader)
    return undefined
  }

  return shader
}

function createCloudRenderer(
  canvas: HTMLCanvasElement,
  diameter: number
): CloudRenderer | undefined {
  const gl = canvas.getContext('webgl', {
    alpha: true,
    antialias: true,
    premultipliedAlpha: true,
  })
  if (!gl) return undefined

  const vertexShader = compileShader(gl, gl.VERTEX_SHADER, VERTEX_SHADER)
  const fragmentShader = compileShader(gl, gl.FRAGMENT_SHADER, FRAGMENT_SHADER)
  const program = vertexShader && fragmentShader ? gl.createProgram() : null
  const buffer = program ? gl.createBuffer() : null

  const abandon = () => {
    if (program) gl.deleteProgram(program)
    if (buffer) gl.deleteBuffer(buffer)
    if (vertexShader) gl.deleteShader(vertexShader)
    if (fragmentShader) gl.deleteShader(fragmentShader)
    return undefined
  }

  if (!vertexShader || !fragmentShader || !program || !buffer) return abandon()

  gl.attachShader(program, vertexShader)
  gl.attachShader(program, fragmentShader)
  gl.linkProgram(program)

  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    console.error('Cloud program link failed:', gl.getProgramInfoLog(program))
    return abandon()
  }

  const positionLocation = gl.getAttribLocation(program, 'a_position')
  if (positionLocation < 0) return abandon()

  const uniforms = {
    resolution: gl.getUniformLocation(program, 'u_resolution'),
    time: gl.getUniformLocation(program, 'u_time'),
    activity: gl.getUniformLocation(program, 'u_activity'),
    deep: gl.getUniformLocation(program, 'u_deep'),
    upper: gl.getUniformLocation(program, 'u_upper'),
    lower: gl.getUniformLocation(program, 'u_lower'),
    milk: gl.getUniformLocation(program, 'u_milk'),
    warp: gl.getUniformLocation(program, 'u_warp'),
    edge: gl.getUniformLocation(program, 'u_edge'),
    saturation: gl.getUniformLocation(program, 'u_saturation'),
  }

  const pixelRatio = Math.min(window.devicePixelRatio || 1, 2)
  const pixelSize = Math.max(1, Math.round(diameter * pixelRatio))
  canvas.width = pixelSize
  canvas.height = pixelSize

  gl.viewport(0, 0, pixelSize, pixelSize)
  gl.useProgram(program)
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer)
  gl.bufferData(
    gl.ARRAY_BUFFER,
    new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
    gl.STATIC_DRAW
  )
  gl.enableVertexAttribArray(positionLocation)
  gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0)
  gl.uniform2f(uniforms.resolution, pixelSize, pixelSize)

  return {
    draw(time, activity, style) {
      // The palette is read as a dark-to-light ramp: densest sky first, lit
      // cloud bank last, which is the order the original's four constants sat in.
      const [deep, upper, lower, milk] = cloudColors(style.palette)

      gl.uniform1f(uniforms.time, time)
      gl.uniform1f(uniforms.activity, activity)
      gl.uniform3f(uniforms.deep, deep[0], deep[1], deep[2])
      gl.uniform3f(uniforms.upper, upper[0], upper[1], upper[2])
      gl.uniform3f(uniforms.lower, lower[0], lower[1], lower[2])
      gl.uniform3f(uniforms.milk, milk[0], milk[1], milk[2])
      gl.uniform1f(uniforms.warp, style.turbulence)
      gl.uniform1f(uniforms.edge, style.softness)
      gl.uniform1f(uniforms.saturation, style.saturation)

      gl.clearColor(0, 0, 0, 0)
      gl.clear(gl.COLOR_BUFFER_BIT)
      gl.drawArrays(gl.TRIANGLES, 0, 6)
    },
    destroy() {
      gl.deleteProgram(program)
      gl.deleteBuffer(buffer)
      gl.deleteShader(vertexShader)
      gl.deleteShader(fragmentShader)
    },
  }
}

function entranceScale(elapsed: number) {
  if (elapsed <= DOT_HOLD_MS) return DOT_SCALE

  if (elapsed <= DOT_HOLD_MS + GROW_MS) {
    const progress = easeOutCubic((elapsed - DOT_HOLD_MS) / GROW_MS)
    return mix(DOT_SCALE, ENTRANCE_OVERSHOOT, progress)
  }

  const settleElapsed = elapsed - DOT_HOLD_MS - GROW_MS
  if (settleElapsed >= SETTLE_MS) return 1

  const progress = clamp(settleElapsed / SETTLE_MS)
  return 1 + (ENTRANCE_OVERSHOOT - 1) * Math.pow(1 - progress, 2)
}

export function Orb({
  style,
  state = 'recording',
  level = 0,
  size,
  paused = false,
  className,
}: OrbProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const launchDotRef = useRef<HTMLSpanElement>(null)
  const spinnerRef = useRef<HTMLSpanElement>(null)

  // Latest props, read by the render loop without restarting it.
  const styleRef = useRef(style)
  const stateRef = useRef(state)
  const levelRef = useRef(level)
  const reducedMotionRef = useRef(false)

  useIsomorphicLayoutEffect(() => {
    styleRef.current = style
    stateRef.current = state
    levelRef.current = level
  }, [style, state, level])

  const box = size ?? style.size
  const diameter = Math.round(box * NEUTRAL_DIAMETER)
  const dotColor = style.palette[0] ?? FALLBACK_PALETTE[0]
  // The densest colour, because the ring sits right on the lit cloud bank.
  const spinnerColor = style.palette[0] ?? FALLBACK_PALETTE[0]

  useEffect(() => {
    const canvas = canvasRef.current
    const launchDot = launchDotRef.current
    const spinner = spinnerRef.current
    if (!canvas || !launchDot || !spinner) return

    const motionQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
    const updateReducedMotion = () => {
      reducedMotionRef.current = motionQuery.matches
    }
    updateReducedMotion()
    motionQuery.addEventListener('change', updateReducedMotion)

    const renderer = createCloudRenderer(canvas, diameter)
    if (!renderer) {
      // No WebGL: a static gradient in the palette's own colours, rather than
      // an empty hole where the popup should be.
      const stops = styleRef.current.palette
      canvas.style.background = `linear-gradient(180deg, ${(stops.length >= 2 ? stops : FALLBACK_PALETTE).join(', ')})`
    }

    let frame = 0
    let previousTime = performance.now()
    let previousState = stateRef.current

    // Only starting a recording earns the dot-and-grow entrance. Processing
    // and finishing mount their own orb, and replaying the entrance on each of
    // those would read as three separate objects rather than one.
    let entranceStarted = previousState === 'recording' ? previousTime : undefined

    let currentLevel = clamp(levelRef.current)
    let currentAudioScale = 1
    let currentOpacity = previousState === 'recording' ? 0 : 1
    let currentExitScale = 1
    let currentSpinnerOpacity = previousState === 'processing' ? 1 : 0
    let flowTime = 0

    const render = (now: number) => {
      const deltaSeconds = Math.min((now - previousTime) / 1000, 0.05)
      previousTime = now

      const current = styleRef.current
      const nextState = stateRef.current
      const reducedMotion = reducedMotionRef.current

      if (nextState !== previousState) {
        if (nextState === 'recording' && previousState === 'idle') entranceStarted = now
        if (nextState === 'dispersing') entranceStarted = undefined
        previousState = nextState
      }

      // Reactivity scales how much of the mic level actually reaches the
      // motion, so the slider spans "barely breathing" to "over-eager".
      const reach = 0.3 + current.reactivity * 1.3
      const rawLevel = clamp(clamp(levelRef.current) * reach)
      const levelRate = rawLevel > currentLevel ? 11 : 6
      currentLevel = reducedMotion ? 0 : damp(currentLevel, rawLevel, levelRate, deltaSeconds)

      // Recording is the original's "listening": the sphere draws in on your
      // voice. Processing is its "speaking" — nothing to listen to, so it only
      // breathes.
      let audioScaleTarget = 1
      if (nextState === 'recording') audioScaleTarget = 1 - currentLevel * LISTEN_SHRINK
      if (nextState === 'processing') audioScaleTarget = 1 + currentLevel * SPEAK_GROW

      const scaleRate = Math.abs(audioScaleTarget - 1) > Math.abs(currentAudioScale - 1) ? 12 : 6
      currentAudioScale = reducedMotion
        ? 1
        : damp(currentAudioScale, audioScaleTarget, scaleRate, deltaSeconds)

      const finishing = nextState === 'dispersing'
      const opacityTarget = finishing ? 0 : 1
      currentOpacity = reducedMotion
        ? opacityTarget
        : damp(currentOpacity, opacityTarget, 14, deltaSeconds)

      // The finish settles inward and fades. It deliberately does not shatter.
      const exitScaleTarget = finishing ? 0.92 : 1
      currentExitScale = reducedMotion
        ? exitScaleTarget
        : damp(currentExitScale, exitScaleTarget, 12, deltaSeconds)

      let scale = currentAudioScale * currentExitScale
      let surfaceMix = 1
      let launchDotOpacity = 0

      if (entranceStarted !== undefined && !reducedMotion) {
        const elapsed = now - entranceStarted
        const entrance = entranceScale(elapsed)
        const audioInfluence = clamp((elapsed - DOT_HOLD_MS - GROW_MS) / SETTLE_MS)
        scale = entrance * mix(1, currentAudioScale, audioInfluence)
        surfaceMix = smoothstepRange(elapsed, SURFACE_FADE_START_MS, SURFACE_FADE_END_MS)
        launchDotOpacity = 1 - smoothstepRange(elapsed, DOT_FADE_START_MS, DOT_FADE_END_MS)
        if (elapsed >= DOT_HOLD_MS + GROW_MS + SETTLE_MS) entranceStarted = undefined
        // The dot is the orb until the surface takes over, so it appears at
        // once instead of fading up with the cloud behind it.
        currentOpacity = finishing ? currentOpacity : 1
      }

      const spinnerTarget = nextState === 'processing' ? 1 : 0
      currentSpinnerOpacity = reducedMotion
        ? spinnerTarget
        : damp(currentSpinnerOpacity, spinnerTarget, 18, deltaSeconds)

      canvas.style.opacity = String(currentOpacity * surfaceMix)
      canvas.style.transform = `scale(${scale})`
      launchDot.style.opacity = String(currentOpacity * launchDotOpacity)
      launchDot.style.transform = `scale(${scale})`
      spinner.style.opacity = String(currentSpinnerOpacity * currentOpacity)
      spinner.style.transform = `rotate(${reducedMotion ? 45 : now * 0.34}deg)`

      // Idle drifts, recording quickens with your voice, processing churns.
      const tempo = 0.4 + current.speed * 1.6
      let speed = 0.24
      let activity = 0.1
      if (nextState === 'recording') {
        speed = 0.72 + currentLevel * 0.78
        activity = 0.28 + currentLevel * 0.32
      } else if (nextState === 'processing') {
        speed = 1.65
        activity = 0.66
      }

      if (!reducedMotion) flowTime += deltaSeconds * speed * tempo
      renderer?.draw(flowTime, activity, current)

      frame = requestAnimationFrame(render)
    }

    if (paused) {
      // A single frame, so an off-screen preview still shows a cloud.
      entranceStarted = undefined
      render(performance.now())
      cancelAnimationFrame(frame)
    } else {
      frame = requestAnimationFrame(render)
    }

    return () => {
      cancelAnimationFrame(frame)
      motionQuery.removeEventListener('change', updateReducedMotion)
      renderer?.destroy()
    }
    // Style fields other than the size are picked up live through styleRef.
  }, [diameter, paused])

  return (
    <div
      className={className}
      style={{
        width: box,
        height: box,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        position: 'relative',
      }}
      aria-hidden
    >
      <span
        style={{
          position: 'relative',
          display: 'block',
          width: diameter,
          height: diameter,
          lineHeight: 0,
        }}
      >
        <span
          ref={launchDotRef}
          style={{
            position: 'absolute',
            inset: 0,
            display: 'block',
            borderRadius: '50%',
            background: dotColor,
            opacity: 0,
            transform: `scale(${DOT_SCALE})`,
            transformOrigin: 'center',
            willChange: 'opacity, transform',
          }}
        />
        <canvas
          ref={canvasRef}
          style={{
            position: 'absolute',
            inset: 0,
            display: 'block',
            width: diameter,
            height: diameter,
            borderRadius: '50%',
            opacity: 0,
            transform: `scale(${DOT_SCALE})`,
            transformOrigin: 'center',
            willChange: 'opacity, transform',
          }}
        />
        <span
          ref={spinnerRef}
          style={{
            position: 'absolute',
            left: '50%',
            top: '50%',
            width: diameter * 0.22,
            height: diameter * 0.22,
            boxSizing: 'border-box',
            border: `${Math.max(1.5, diameter * 0.026)}px solid rgba(0, 0, 0, 0.14)`,
            borderTopColor: spinnerColor,
            borderRightColor: spinnerColor,
            borderRadius: '50%',
            opacity: 0,
            transform: 'rotate(0deg)',
            transformOrigin: 'center',
            marginLeft: diameter * -0.11,
            marginTop: diameter * -0.11,
            willChange: 'opacity, transform',
          }}
        />
      </span>
    </div>
  )
}
