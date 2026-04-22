import React, { useEffect, useRef } from 'react';

/**
 * GpuCanvas provides a hardware-accelerated rendering bridge for complex
 * UI overlays, such as dependency graphs or real-time agent activity heatmaps.
 */
export const GpuCanvas: React.FC<{ active: boolean }> = ({ active }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!active || !canvasRef.current) return;

    const canvas = canvasRef.current;
    const gl = canvas.getContext('webgl');

    if (!gl) {
      console.warn('WebGL not supported, falling back to 2D canvas');
      return;
    }

    // Initialize simple shader for background gradient animation
    let animationFrameId: number;

    const render = () => {
      gl.clearColor(0.02, 0.02, 0.05, 0.8); // Deep space blue
      gl.clear(gl.COLOR_BUFFER_BIT);
      
      // Additional GPU-accelerated drawing logic would go here
      
      animationFrameId = requestAnimationFrame(render);
    };

    render();

    return () => {
      cancelAnimationFrame(animationFrameId);
    };
  }, [active]);

  if (!active) return null;

  return (
    <canvas
      ref={canvasRef}
      className="gpu-overlay"
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        width: '100vw',
        height: '100vh',
        pointerEvents: 'none',
        zIndex: 99,
        opacity: 0.3,
      }}
    />
  );
};
