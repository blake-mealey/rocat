import clsx from 'clsx';

export function Columns({
  count,
  children,
}: {
  count: 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12;
  children: React.ReactNode;
}) {
  return (
    <div
      className={clsx('grid gridcols', {
        'grid-cols-1': count === 1,
        'grid-cols-2': count === 2,
        'grid-cols-3': count === 3,
        'grid-cols-4': count === 4,
        'grid-cols-5': count === 5,
        'grid-cols-6': count === 6,
        'grid-cols-7': count === 7,
        'grid-cols-8': count === 8,
        'grid-cols-9': count === 9,
        'grid-cols-10': count === 10,
        'grid-cols-11': count === 11,
        'grid-cols-12': count === 12,
      })}
    >
      {children}
    </div>
  );
}

export function Column({
  children,
  heading,
}: {
  children: React.ReactNode;
  heading?: string;
}) {
  return (
    <div>
      {heading !== undefined ? (
        // styles copied from nextra h5
        <h5 className="nx-font-semibold nx-tracking-tight nx-text-slate-900 dark:nx-text-slate-100 nx-mt-8 nx-text-lg">
          {heading}
        </h5>
      ) : null}
      {children}
    </div>
  );
}
