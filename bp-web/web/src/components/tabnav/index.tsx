import React from 'react';
import { BaseProps } from '../common';

interface TabItem {
  name: string;
  title: string;
}

interface TabNavProps extends BaseProps {
  items: TabItem[];
  current: string;
  onChange: (item: string) => void;
}

export const TabNav: React.FC<TabNavProps> = (props) => {
  const {
    className,
    style,
    items,
    current,
    onChange,
    children,
  } = props;

  return (
    <div className={className} style={style}>
      <div className="tabnav mb-0">
        <nav className="tabnav-tabs" aria-label="">
          {items.map(item => (
            <a
              key={item.name}
              className="tabnav-tab"
              style={{
                'fontWeight': item.name === current ? '600' : 'normal',
              }}
              href={`#${item.name}`}
              aria-current={item.name === current}
              onClick={() => onChange(item.name)}
            >
              {item.title}
            </a>
          ))}
        </nav>
      </div>
      <div className="p-3 border border-top-0 rounded-bottom-2">
        {React.Children.map(children, (child: any) => {
          // console.log({ child });
          // if (child.type.name !== 'Nav') {
          //   console.warn('children of TabNav must be TabNav.Nav');
          //   return null;
          // }
          if (child.props.name !== current) {
            return null;
          }
          return child;
        })}
      </div>
    </div>
  );
};

interface NavProps {
  name: string;
}

export const TabNavItem: React.FC<NavProps> = (props) => {
  const { children } = props;
  return <React.Fragment>{children}</React.Fragment>;
};
