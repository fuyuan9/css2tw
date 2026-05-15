import React from 'react';
import clsx from 'clsx';
import { cva } from 'class-variance-authority';
import classNames from 'classnames';

const button = cva('base-button', {
  variants: {
    intent: {
      primary: 'btn-primary',
      secondary: 'btn-secondary',
    },
  },
});

export const MyComponent = ({ isActive, size }: { isActive: boolean, size: string }) => {
  const dynamicClasses = 'some-class ' + (isActive ? 'active' : 'inactive');

  return (
    <div>
      {/* clsx library */}
      <button className={clsx('btn', isActive && 'btn-active', { 'btn-large': size === 'lg' })}>
        Click me
      </button>

      {/* classnames library */}
      <div className={classNames('container', { 'bg-red': isActive })}>
        Content
      </div>

      {/* cva (class-variance-authority) */}
      <div className={button({ intent: 'primary' })}>
        CVA Button
      </div>

      {/* Template literals */}
      <span className={`text-${size} font-bold ${isActive ? 'visible' : 'hidden'}`}>
        Status
      </span>

      {/* Ternary expressions */}
      <div className={isActive ? 'is-active' : 'is-inactive'}>
        Ternary
      </div>

      {/* Logical && patterns */}
      <p className={isActive && 'bold-text'}>
        Logical AND
      </p>

      {/* Array join patterns */}
      <div className={['item', size].join(' ')}>
        Array join
      </div>

      {/* Computed object keys */}
      <div className={{ [size]: true, 'custom': true } as any}>
        Computed
      </div>

      {/* Variable reference */}
      <div className={dynamicClasses}>
        Variable
      </div>
    </div>
  );
};
