interface LoadingIndicatorProps {
  label?: string;
  size?: "sm" | "md" | "lg";
  overlay?: boolean;
  className?: string;
}

export function LoadingIndicator({
  label,
  size = "md",
  overlay = false,
  className,
}: LoadingIndicatorProps): JSX.Element {
  const classes = [
    "loading-indicator",
    `size-${size}`,
    overlay ? "overlay" : "",
    className ?? "",
  ]
    .filter((value) => value.length > 0)
    .join(" ");

  return (
    <div className={classes} role="status" aria-live="polite" aria-busy="true">
      <div className="loading-indicator-content">
        <span className="loading-spinner" aria-hidden="true" />
        {label ? <span className="loading-label">{label}</span> : null}
      </div>
    </div>
  );
}
