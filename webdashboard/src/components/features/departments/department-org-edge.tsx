import { BaseEdge, getSmoothStepPath, Position, type EdgeProps } from "@xyflow/react";

// Custom Edge component for Orthogonal routing
export const OrthoEdge = ({
  markerEnd,
  sourcePosition,
  sourceX,
  sourceY,
  style = {},
  targetPosition,
  targetX,
  targetY,
}: EdgeProps) => {
  let path = "";

  if (sourcePosition === Position.Bottom && targetPosition === Position.Left) {
    const yGap = sourceY + 40;
    const xGutter = targetX - 20;
    path = `M ${sourceX} ${sourceY} L ${sourceX} ${yGap} L ${xGutter} ${yGap} L ${xGutter} ${targetY} L ${targetX} ${targetY}`;
  } else if (sourcePosition === Position.Left && targetPosition === Position.Left) {
    const xGutter = sourceX - 20;
    path = `M ${sourceX} ${sourceY} L ${xGutter} ${sourceY} L ${xGutter} ${targetY} L ${targetX} ${targetY}`;
  } else {
    const [smoothStepPath] = getSmoothStepPath({
      borderRadius: 8,
      sourcePosition,
      sourceX,
      sourceY,
      targetPosition,
      targetX,
      targetY,
    });
    path = smoothStepPath;
  }

  return (
    <BaseEdge markerEnd={markerEnd} path={path} style={style} />
  );
};
