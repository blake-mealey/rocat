import { MDXRemote } from 'next-mdx-remote';
import { Callout, Tab, Tabs } from 'nextra-theme-docs';
import { useSSG } from 'nextra/data';
import { useMDXComponents } from 'nextra/mdx';

export function DynamicMarkdown({ name }: { name: string }) {
  const { dynamicMarkdown } = useSSG() as {
    dynamicMarkdown: Record<string, any>;
  };
  const components = useMDXComponents();

  const content = dynamicMarkdown[name];
  if (!content) {
    return null;
  }

  return (
    <MDXRemote
      compiledSource={content}
      components={{ ...components, Callout, Tabs, Tab }}
    />
  );
}
