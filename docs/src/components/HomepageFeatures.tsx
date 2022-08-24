/**
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import React from 'react';
import clsx from 'clsx';
import styles from './HomepageFeatures.module.css';

type FeatureItem = {
  title: string;
  description: JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Easy to Use',
    description: (
      <>
        OT is easy to use LISP-like language, that can reuse your existing codebase using FFI or can 
        be embeded into your applications.
      </>
    ),
  },
  {
    title: 'Focus on What Matters',
    description: (
      <>
        OT is dynamic language and has easy to learn syntax, because of that you can focus on developing
        your idea and not about the language itself. 
      </>
    ),
  },
  {
    title: 'Pattern matching',
    description: (
      <>
        As a functional language OT has a great pattern matching using function signatures.
      </>
    ),
  },
];

function Feature({title, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center padding-horiz--md">
        <h3>{title}</h3>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
