import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { open } from "@tauri-apps/plugin-shell";
import type { AnchorHTMLAttributes } from "react";

interface MarkdownPreviewProps {
  content: string;
}

function ExternalLink(props: AnchorHTMLAttributes<HTMLAnchorElement>) {
  const handleClick = (e: React.MouseEvent<HTMLAnchorElement>) => {
    e.preventDefault();
    if (props.href) {
      open(props.href);
    }
  };
  return (
    <a {...props} onClick={handleClick} />
  );
}

export function MarkdownPreview({ content }: MarkdownPreviewProps) {
  return (
    <div className="markdown-preview">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{ a: ExternalLink }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}
