import { SkeletonItem } from "@fluentui/react-components";
import Image, { ImageProps } from "next/image";
import React, { useState } from "react";

interface OnlineImageProps extends Omit<ImageProps, "src" | "alt"> {
  src: string | string[] | undefined;
  alt?: string;
  className?: string;
  aspectRatio?: string;
  placeholderSrc?: string;
  loadingNode?: React.ReactNode;
  style?: React.CSSProperties;
}

const OnlineImage: React.FC<OnlineImageProps> = ({
  src,
  alt = "",
  className,
  aspectRatio = "3/2",
  placeholderSrc = "/res.png",
  loadingNode = (
      <SkeletonItem style={{ width: "100%", height: "100%" }} />
  ),
  style,
  ...props
}) => {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  const imageSrc = error
    ? placeholderSrc
    : Array.isArray(src)
    ? src[0] ?? ""
      : src ?? "";

  return (
    <div
      className={className}
      style={{
        position: "relative",
        width: " calc(100% - 8px)",
        margin: "4px 4px 0 4px",
        aspectRatio,
        ...style,    // 支持外部 style 覆盖
      }}
    >
      {loading && (
        <div
          style={{
            position: "absolute",
            inset: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 0,
            background: "transparent",
          }}
        >
          {loadingNode}
        </div>
      )}
      <Image
        src={imageSrc}
        alt={alt}
        fill
        loading="eager"
        style={{ objectFit: "cover" }}
        onLoad={() => setLoading(false)}
        onError={e => {
          setError(true);
          setLoading(false);
        }}
        priority
        {...props}
      />
    </div>
  );
};

export default OnlineImage;