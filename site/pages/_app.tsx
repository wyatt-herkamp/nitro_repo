import React, { Component } from 'react';
import Head from 'next/head';
import { AppProps } from 'next/app';
export default function MyApp(props: AppProps) {
  const { Component, pageProps } = props;


  return (
    <React.Fragment>
      <Head>
        <meta name="viewport" content="minimum-scale=1, initial-scale=1, width=device-width" />

      </Head>
        <Component {...pageProps} />


    </React.Fragment>
  );
}