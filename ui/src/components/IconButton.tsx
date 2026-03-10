import clsx from "clsx";
import type { ButtonHTMLAttributes } from "react";
import type { LucideIcon } from "lucide-react";

interface IconButtonBaseProps {
  icon: LucideIcon;
  label: string;
  variant?: "primary" | "secondary" | "danger";
  className?: string;
}

type IconButtonProps = IconButtonBaseProps & ButtonHTMLAttributes<HTMLButtonElement>;

export function IconButton({
  icon: Icon,
  label,
  variant = "secondary",
  className,
  ...props
}: IconButtonProps): JSX.Element {
  return (
    <button
      type="button"
      className={clsx("btn icon-btn", variantClassName(variant), className)}
      aria-label={label}
      title={label}
      {...props}
    >
      <Icon size={16} strokeWidth={2} aria-hidden="true" />
      <span className="sr-only">{label}</span>
    </button>
  );
}

function variantClassName(variant: "primary" | "secondary" | "danger"): string {
  if (variant === "primary") {
    return "";
  }
  if (variant === "danger") {
    return "danger";
  }
  return "secondary";
}
