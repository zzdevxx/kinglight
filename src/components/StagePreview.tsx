import { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { beamDirection, fixturePosition } from "../lib/stage";
import type { FixtureSnapshot } from "../types";
import "./StagePreview.css";

interface StagePreviewProps {
  fixtures: FixtureSnapshot[];
  armed: boolean;
  blackout: boolean;
}

interface CameraState {
  yaw: number;
  pitch: number;
  radius: number;
}

const FRONT = new THREE.Vector3(0, 0, 1);
const CAMERA_DEFAULT: CameraState = { yaw: 0, pitch: 0.03, radius: 18 };
const BEAM_LENGTH = 24;
const BEAM_WIDTH = 0.88;

function createBeamMaterial(): THREE.ShaderMaterial {
  return new THREE.ShaderMaterial({
    uniforms: {
      beamColor: { value: new THREE.Color(1, 1, 1) },
      level: { value: 0 },
    },
    vertexShader: `
      varying vec3 vNormal;
      varying vec3 vView;
      varying float vAlong;
      void main() {
        vec4 viewPosition = modelViewMatrix * vec4(position, 1.0);
        vNormal = normalize(normalMatrix * normal);
        vView = normalize(-viewPosition.xyz);
        vAlong = position.z;
        gl_Position = projectionMatrix * viewPosition;
      }
    `,
    fragmentShader: `
      uniform vec3 beamColor;
      uniform float level;
      varying vec3 vNormal;
      varying vec3 vView;
      varying float vAlong;
      void main() {
        float facing = abs(dot(normalize(vNormal), normalize(vView)));
        float edge = smoothstep(0.0, 0.7, facing);
        float haze = mix(1.0, 0.36, smoothstep(0.0, 1.0, vAlong));
        float tail = 1.0 - smoothstep(0.82, 1.0, vAlong);
        gl_FragColor = vec4(beamColor, level * edge * haze * tail);
      }
    `,
    transparent: true,
    depthTest: false,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
    side: THREE.DoubleSide,
    toneMapped: false,
  });
}

export function StagePreview({ fixtures, armed, blackout }: StagePreviewProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const fixturesRef = useRef(fixtures);
  const pausedRef = useRef(false);
  const cameraRef = useRef<CameraState>({ ...CAMERA_DEFAULT });
  const [paused, setPaused] = useState(false);
  const [rendererError, setRendererError] = useState<string | null>(null);
  fixturesRef.current = fixtures;
  pausedRef.current = paused;

  useEffect(() => {
    const host = hostRef.current;
    const canvas = canvasRef.current;
    if (!host || !canvas) return;

    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: "high-performance" });
    } catch (error) {
      setRendererError(error instanceof Error ? error.message : "WebGL tidak tersedia.");
      return;
    }
    renderer.setClearColor(0x000000, 1);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 1.5));
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(48, 1, 0.1, 90);
    const target = new THREE.Vector3(0, 1.7, -3);

    const geometry = new THREE.ConeGeometry(1, 1, 40, 1, true);
    geometry.translate(0, -0.5, 0);
    geometry.rotateX(-Math.PI / 2);
    const lampGeometry = new THREE.SphereGeometry(0.024, 8, 6);
    const lamps: THREE.Mesh<THREE.SphereGeometry, THREE.MeshBasicMaterial>[] = [];
    const beams: THREE.Group[] = [];
    const materials: THREE.ShaderMaterial[] = [];

    for (let index = 0; index < 16; index += 1) {
      const group = new THREE.Group();
      group.position.set(...fixturePosition(index));
      for (const width of [1, 0.72, 0.4]) {
        const material = createBeamMaterial();
        const mesh = new THREE.Mesh(geometry, material);
        mesh.scale.set(BEAM_WIDTH * width, BEAM_WIDTH * width, BEAM_LENGTH);
        group.add(mesh);
        materials.push(material);
      }
      scene.add(group);
      beams.push(group);
      const lamp = new THREE.Mesh(lampGeometry, new THREE.MeshBasicMaterial({ color: 0xffffff, toneMapped: false }));
      lamp.position.copy(group.position);
      scene.add(lamp);
      lamps.push(lamp);
    }

    const resize = () => {
      const width = Math.max(1, host.clientWidth);
      const height = Math.max(1, host.clientHeight);
      renderer.setSize(width, height, false);
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
    };
    const observer = new ResizeObserver(resize);
    observer.observe(host);
    resize();

    const pointer = { id: -1, x: 0, y: 0 };
    const onPointerDown = (event: PointerEvent) => {
      if (event.button !== 0) return;
      pointer.id = event.pointerId;
      pointer.x = event.clientX;
      pointer.y = event.clientY;
      canvas.setPointerCapture(event.pointerId);
    };
    const onPointerMove = (event: PointerEvent) => {
      if (pointer.id !== event.pointerId) return;
      const state = cameraRef.current;
      state.yaw -= (event.clientX - pointer.x) * 0.004;
      state.pitch = THREE.MathUtils.clamp(state.pitch + (event.clientY - pointer.y) * 0.004, -0.55, 1.2);
      pointer.x = event.clientX;
      pointer.y = event.clientY;
    };
    const onPointerUp = (event: PointerEvent) => {
      if (pointer.id !== event.pointerId) return;
      pointer.id = -1;
      if (canvas.hasPointerCapture(event.pointerId)) canvas.releasePointerCapture(event.pointerId);
    };
    const onWheel = (event: WheelEvent) => {
      event.preventDefault();
      cameraRef.current.radius = THREE.MathUtils.clamp(cameraRef.current.radius + event.deltaY * 0.01, 10, 32);
    };
    let contextLost = false;
    const onContextLost = (event: Event) => {
      event.preventDefault();
      contextLost = true;
      setRendererError("Preview terhenti. Buka ulang aplikasi jika tampilan tidak kembali.");
    };
    const onContextRestored = () => {
      contextLost = false;
      setRendererError(null);
      resize();
    };
    canvas.addEventListener("pointerdown", onPointerDown);
    canvas.addEventListener("pointermove", onPointerMove);
    canvas.addEventListener("pointerup", onPointerUp);
    canvas.addEventListener("pointercancel", onPointerUp);
    canvas.addEventListener("wheel", onWheel, { passive: false });
    canvas.addEventListener("webglcontextlost", onContextLost);
    canvas.addEventListener("webglcontextrestored", onContextRestored);

    const direction = new THREE.Vector3();
    let animationFrame = 0;
    const render = () => {
      animationFrame = requestAnimationFrame(render);
      if (contextLost || pausedRef.current || document.hidden) return;
      const state = cameraRef.current;
      camera.position.set(
        target.x + Math.sin(state.yaw) * Math.cos(state.pitch) * state.radius,
        target.y + Math.sin(state.pitch) * state.radius,
        target.z + Math.cos(state.yaw) * Math.cos(state.pitch) * state.radius,
      );
      camera.lookAt(target);
      for (let index = 0; index < beams.length; index += 1) {
        const fixture = fixturesRef.current[index];
        const visible = Boolean(fixture && fixture.dimmer > 0.001);
        beams[index].visible = visible;
        lamps[index].visible = visible;
        if (!fixture || !visible) continue;
        direction.set(...beamDirection(fixture.pan, fixture.tilt));
        beams[index].quaternion.setFromUnitVectors(FRONT, direction);
        const level = THREE.MathUtils.clamp(fixture.dimmer, 0, 1);
        for (let layer = 0; layer < 3; layer += 1) {
          const material = materials[index * 3 + layer];
          material.uniforms.beamColor.value.setRGB(...fixture.color);
          material.uniforms.level.value = level * [0.12, 0.15, 0.2][layer];
        }
        lamps[index].material.color.setRGB(...fixture.color);
      }
      renderer.render(scene, camera);
    };
    render();

    return () => {
      cancelAnimationFrame(animationFrame);
      observer.disconnect();
      canvas.removeEventListener("pointerdown", onPointerDown);
      canvas.removeEventListener("pointermove", onPointerMove);
      canvas.removeEventListener("pointerup", onPointerUp);
      canvas.removeEventListener("pointercancel", onPointerUp);
      canvas.removeEventListener("wheel", onWheel);
      canvas.removeEventListener("webglcontextlost", onContextLost);
      canvas.removeEventListener("webglcontextrestored", onContextRestored);
      materials.forEach((material) => material.dispose());
      lamps.forEach((lamp) => lamp.material.dispose());
      lampGeometry.dispose();
      geometry.dispose();
      renderer.dispose();
    };
  }, []);

  return (
    <section className="stage-panel" aria-label="Panggung simulator">
      <div className="stage-surface" ref={hostRef}>
        <canvas ref={canvasRef} className="stage-canvas" aria-label="Preview beam lighting" />
      </div>
      <details className="stage-view-menu">
        <summary>Tampilan</summary>
        <div className="stage-view-options">
          <button type="button" onClick={() => { cameraRef.current = { ...CAMERA_DEFAULT }; }}>Depan</button>
          <button type="button" onClick={() => { cameraRef.current = { yaw: 0.9, pitch: 0.28, radius: 20 }; }}>Samping</button>
          <button type="button" onClick={() => { cameraRef.current = { yaw: 0, pitch: 1.1, radius: 22 }; }}>Atas</button>
          <button type="button" aria-pressed={paused} onClick={() => setPaused((value) => !value)}>
            {paused ? "Lanjutkan preview" : "Jeda preview"}
          </button>
        </div>
      </details>
      {blackout || paused ? (
        <div className={`stage-state-flag${blackout ? " stage-state-blackout" : ""}`}>
          {blackout ? "BLACK OUT" : ""}{blackout && paused ? " / " : ""}
          {paused ? `Preview dijeda${armed ? ". Art-Net tetap mengirim." : ""}` : ""}
        </div>
      ) : null}
      {rendererError ? <p className="stage-render-error" role="alert">{rendererError}</p> : null}
    </section>
  );
}
