import { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { clamp } from "../lib/format";
import type { FixtureSnapshot } from "../types";

type CameraPreset = "front" | "side" | "top" | "reset";
type Quality = "standard" | "high";

interface StagePreviewProps {
  fixtures: FixtureSnapshot[];
  armed: boolean;
  blackout: boolean;
}

interface CameraState {
  theta: number;
  phi: number;
  radius: number;
}

const FIXTURE_COUNT = 16;
const MOUNT_HEIGHT = 6.6;
const DOWN = new THREE.Vector3(0, -1, 0);

function mountPosition(index: number): [number, number] {
  const row = index < 8 ? 0 : 1;
  const column = index % 8;
  return [(column - 3.5) * 2.1, row === 0 ? -2.6 : 2.6];
}

export function StagePreview({ fixtures, armed, blackout }: StagePreviewProps) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const resizeRef = useRef<() => void>(() => undefined);
  const fixturesRef = useRef(fixtures);
  const pausedRef = useRef(false);
  const cameraRef = useRef<CameraState>({ theta: 0, phi: 1.04, radius: 18 });

  const [paused, setPaused] = useState(false);
  const [quality, setQuality] = useState<Quality>("standard");
  const [rendererError, setRendererError] = useState<string | null>(null);

  fixturesRef.current = fixtures;
  pausedRef.current = paused;

  useEffect(() => {
    const host = hostRef.current;
    const canvas = canvasRef.current;
    if (!host || !canvas) return;

    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({
        canvas,
        antialias: true,
        alpha: false,
        powerPreference: "high-performance",
      });
    } catch (error) {
      setRendererError(
        error instanceof Error ? error.message : "WebGL renderer unavailable",
      );
      return;
    }

    rendererRef.current = renderer;
    renderer.setClearColor(0x0c0f14, 1);
    renderer.setPixelRatio(1);

    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x0c0f14);
    const camera = new THREE.PerspectiveCamera(52, 1, 0.1, 200);
    const target = new THREE.Vector3(0, 2.2, 0);

    const floorGeometry = new THREE.PlaneGeometry(34, 22);
    const floorMaterial = new THREE.MeshStandardMaterial({
      color: 0x151a21,
      roughness: 0.94,
      metalness: 0.02,
    });
    const floor = new THREE.Mesh(floorGeometry, floorMaterial);
    floor.rotation.x = -Math.PI / 2;
    scene.add(floor);

    const grid = new THREE.GridHelper(34, 17, 0x2c3540, 0x1c232c);
    grid.position.y = 0.012;
    scene.add(grid);

    scene.add(new THREE.AmbientLight(0x8ba0b8, 0.5));
    const keyLight = new THREE.DirectionalLight(0xaebfd2, 0.7);
    keyLight.position.set(6, 14, 9);
    scene.add(keyLight);

    const bodyGeometry = new THREE.BoxGeometry(0.42, 0.24, 0.42);
    const bodyMaterial = new THREE.MeshStandardMaterial({
      color: 0x3c4652,
      roughness: 0.6,
      metalness: 0.25,
    });
    const bodies = new THREE.InstancedMesh(
      bodyGeometry,
      bodyMaterial,
      FIXTURE_COUNT,
    );
    const matrix = new THREE.Matrix4();
    for (let index = 0; index < FIXTURE_COUNT; index += 1) {
      const [x, z] = mountPosition(index);
      matrix.makeTranslation(x, MOUNT_HEIGHT, z);
      bodies.setMatrixAt(index, matrix);
    }
    bodies.instanceMatrix.needsUpdate = true;
    scene.add(bodies);

    const beamGeometry = new THREE.ConeGeometry(0.62, 1, 20, 1, true);
    beamGeometry.translate(0, -0.5, 0);
    const discGeometry = new THREE.CircleGeometry(1, 28);

    const beams: THREE.Mesh[] = [];
    const discs: THREE.Mesh[] = [];

    for (let index = 0; index < FIXTURE_COUNT; index += 1) {
      const [x, z] = mountPosition(index);
      const beamMaterial = new THREE.ShaderMaterial({
        uniforms: {
          uColor: { value: new THREE.Color(1, 1, 1) },
          uOpacity: { value: 0 },
        },
        vertexShader: `
          varying float vAlong;
          void main() {
            vAlong = clamp(-position.y, 0.0, 1.0);
            gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
          }
        `,
        fragmentShader: `
          uniform vec3 uColor;
          uniform float uOpacity;
          varying float vAlong;
          void main() {
            float leadIn = smoothstep(0.0, 0.14, vAlong);
            float tail = 1.0 - 0.5 * smoothstep(0.7, 1.0, vAlong);
            gl_FragColor = vec4(uColor, uOpacity * leadIn * tail);
          }
        `,
        transparent: true,
        depthWrite: false,
        blending: THREE.AdditiveBlending,
        side: THREE.DoubleSide,
      });
      const beam = new THREE.Mesh(beamGeometry, beamMaterial);
      beam.position.set(x, MOUNT_HEIGHT, z);
      beam.visible = false;
      beams.push(beam);
      scene.add(beam);

      const discMaterial = new THREE.MeshBasicMaterial({
        color: 0xffffff,
        transparent: true,
        opacity: 0,
        depthWrite: false,
        blending: THREE.AdditiveBlending,
        side: THREE.DoubleSide,
      });
      const disc = new THREE.Mesh(discGeometry, discMaterial);
      disc.rotation.x = -Math.PI / 2;
      disc.visible = false;
      discs.push(disc);
      scene.add(disc);
    }

    const resize = () => {
      const width = Math.max(1, host.clientWidth);
      const height = Math.max(1, host.clientHeight);
      renderer.setSize(width, height, false);
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
    };
    resizeRef.current = resize;
    resize();

    const observer =
      typeof ResizeObserver === "undefined" ? undefined : new ResizeObserver(resize);
    observer?.observe(host);

    const drag = { active: false, x: 0, y: 0, pointerId: -1 };
    const dom = renderer.domElement;

    const onPointerDown = (event: PointerEvent) => {
      drag.active = true;
      drag.x = event.clientX;
      drag.y = event.clientY;
      drag.pointerId = event.pointerId;
      dom.setPointerCapture(event.pointerId);
    };
    const onPointerMove = (event: PointerEvent) => {
      if (!drag.active || event.pointerId !== drag.pointerId) return;
      const dx = event.clientX - drag.x;
      const dy = event.clientY - drag.y;
      drag.x = event.clientX;
      drag.y = event.clientY;
      const state = cameraRef.current;
      state.theta -= dx * 0.006;
      state.phi = clamp(state.phi + dy * 0.006, 0.18, 1.48);
    };
    const onPointerUp = (event: PointerEvent) => {
      if (event.pointerId !== drag.pointerId) return;
      drag.active = false;
      if (dom.hasPointerCapture(event.pointerId)) {
        dom.releasePointerCapture(event.pointerId);
      }
    };
    const onWheel = (event: WheelEvent) => {
      event.preventDefault();
      const state = cameraRef.current;
      state.radius = clamp(state.radius + event.deltaY * 0.012, 8, 34);
    };

    dom.addEventListener("pointerdown", onPointerDown);
    dom.addEventListener("pointermove", onPointerMove);
    dom.addEventListener("pointerup", onPointerUp);
    dom.addEventListener("pointercancel", onPointerUp);
    dom.addEventListener("wheel", onWheel, { passive: false });

    const direction = new THREE.Vector3();
    let animationFrame = 0;

    const render = () => {
      animationFrame = requestAnimationFrame(render);
      const state = cameraRef.current;
      const sinPhi = Math.sin(state.phi);
      camera.position.set(
        target.x + state.radius * sinPhi * Math.sin(state.theta),
        target.y + state.radius * Math.cos(state.phi),
        target.z + state.radius * sinPhi * Math.cos(state.theta),
      );
      camera.lookAt(target);

      const list = fixturesRef.current;
      for (let index = 0; index < FIXTURE_COUNT; index += 1) {
        const fixture = list[index];
        const beam = beams[index];
        const disc = discs[index];
        if (!fixture || fixture.dimmer <= 0.001) {
          beam.visible = false;
          disc.visible = false;
          continue;
        }

        const level = clamp(fixture.dimmer, 0, 1);
        const pan = (fixture.pan - 0.5) * Math.PI * 1.25;
        const tilt = (fixture.tilt - 0.5) * Math.PI * 0.8;
        const horizontal = Math.sin(tilt);
        direction.set(
          Math.sin(pan) * horizontal,
          -Math.cos(tilt),
          Math.cos(pan) * horizontal,
        ).normalize();

        let length = 7;
        let hitsFloor = false;
        if (direction.y < -0.06) {
          length = Math.min(MOUNT_HEIGHT / -direction.y, 20);
          hitsFloor = true;
        }

        const beamMaterial = beam.material as THREE.ShaderMaterial;
        beamMaterial.uniforms.uColor.value.setRGB(
          clamp(fixture.color[0] ?? 0, 0, 1),
          clamp(fixture.color[1] ?? 0, 0, 1),
          clamp(fixture.color[2] ?? 0, 0, 1),
        );
        beamMaterial.uniforms.uOpacity.value = 0.05 + level * 0.22;
        beam.quaternion.setFromUnitVectors(DOWN, direction);
        beam.scale.set(1, length, 1);
        beam.visible = true;

        if (hitsFloor) {
          const discMaterial = disc.material as THREE.MeshBasicMaterial;
          discMaterial.color.setRGB(
            clamp(fixture.color[0] ?? 0, 0, 1),
            clamp(fixture.color[1] ?? 0, 0, 1),
            clamp(fixture.color[2] ?? 0, 0, 1),
          );
          discMaterial.opacity = 0.08 + level * 0.42;
          disc.position.set(
            beam.position.x + direction.x * length,
            0.018,
            beam.position.z + direction.z * length,
          );
          const radius = 0.5 + level * 0.55;
          disc.scale.set(radius, radius, 1);
          disc.visible = true;
        } else {
          disc.visible = false;
        }
      }

      if (!pausedRef.current) renderer.render(scene, camera);
    };
    render();

    return () => {
      cancelAnimationFrame(animationFrame);
      observer?.disconnect();
      dom.removeEventListener("pointerdown", onPointerDown);
      dom.removeEventListener("pointermove", onPointerMove);
      dom.removeEventListener("pointerup", onPointerUp);
      dom.removeEventListener("pointercancel", onPointerUp);
      dom.removeEventListener("wheel", onWheel);
      beams.forEach((beam) => (beam.material as THREE.Material).dispose());
      discs.forEach((disc) => (disc.material as THREE.Material).dispose());
      beamGeometry.dispose();
      discGeometry.dispose();
      bodyGeometry.dispose();
      bodyMaterial.dispose();
      floorGeometry.dispose();
      floorMaterial.dispose();
      grid.geometry.dispose();
      (grid.material as THREE.Material).dispose();
      renderer.dispose();
      rendererRef.current = null;
      resizeRef.current = () => undefined;
    };
  }, []);

  useEffect(() => {
    const renderer = rendererRef.current;
    if (!renderer) return;
    const ratio = quality === "high" ? Math.min(window.devicePixelRatio || 1, 2) : 1;
    renderer.setPixelRatio(ratio);
    resizeRef.current();
  }, [quality]);

  const setView = (preset: CameraPreset) => {
    const state = cameraRef.current;
    if (preset === "side") {
      state.theta = Math.PI / 2;
      state.phi = 1.04;
      state.radius = 18;
    } else if (preset === "top") {
      state.theta = 0;
      state.phi = 0.2;
      state.radius = 21;
    } else {
      state.theta = 0;
      state.phi = 1.04;
      state.radius = 18;
    }
  };

  return (
    <section className="panel stage-panel" aria-labelledby="stage-title">
      <div className="panel-head">
        <div>
          <h2 id="stage-title" className="panel-title">Stage preview</h2>
          <p className="panel-sub">Frame-driven beams. Drag to orbit, wheel to zoom.</p>
        </div>
        <div className="stage-tools">
          <button type="button" className="btn" onClick={() => setView("front")}>Front</button>
          <button type="button" className="btn" onClick={() => setView("side")}>Side</button>
          <button type="button" className="btn" onClick={() => setView("top")}>Top</button>
          <button type="button" className="btn" onClick={() => setView("reset")}>Reset</button>
          <button
            type="button"
            className="btn"
            aria-pressed={quality === "high"}
            onClick={() => setQuality((current) => (current === "high" ? "standard" : "high"))}
          >
            Quality: {quality === "high" ? "High" : "Standard"}
          </button>
          <button
            type="button"
            className="btn btn-primary"
            aria-pressed={paused}
            onClick={() => setPaused((current) => !current)}
          >
            {paused ? "Resume Preview" : "Pause Preview"}
          </button>
        </div>
      </div>

      <div className="stage-surface" ref={hostRef}>
        <canvas ref={canvasRef} className="stage-canvas" aria-label="Stage preview" />
        {blackout ? <div className="stage-flag stage-flag-danger">Blackout active</div> : null}
        {paused ? (
          <div className="stage-flag" role="status">
            {armed ? "Preview paused. Output continues." : "Preview paused. Output remains disarmed."}
          </div>
        ) : null}
        {rendererError ? (
          <div className="stage-fallback" role="alert">
            Stage renderer could not start: {rendererError}
          </div>
        ) : null}
      </div>
    </section>
  );
}
