import React, { useState } from 'react';
import Head from 'next/head';
import { AppProps } from 'next/app';
import { ThemeProvider } from '@material-ui/core/styles';
import CssBaseline from '@material-ui/core/CssBaseline';
import theme from '../src/theme';
import MyNavBar from '../src/IndexNavBar';
import { ToastContainer } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';
import '../styles/global.css'
import { Cookies } from 'react-cookie';
import axios from 'axios';
import useSWR from 'swr';
import { API_URL } from '../src/config';
const fetcher = (url, token) =>
  axios
    .get(url, { headers: { Authorization: "Bearer " + token } })
    .then((res) => res.data);
export default function MyApp(props: AppProps) {
  const { Component, pageProps } = props;

  React.useEffect(() => {
    // Remove the server-side injected CSS.
    const jssStyles = document.querySelector('#jss-server-side');
    if (jssStyles) {
      jssStyles.parentElement!.removeChild(jssStyles);
    }
  }, []);
  let auth = false;
  if (!auth) {
    const cookies = new Cookies();
    let token = cookies.get("auth_token");
    if (token != undefined) {
      const { data, error } = useSWR([API_URL + "/api/me", token], fetcher)
      if (!error && data) {
        console.log("logged");
        auth = true;
      }
    }
  }
  return (
    <React.Fragment>
      <Head>
        <title>Nitro Repo</title>
        <meta name="viewport" content="minimum-scale=1, initial-scale=1, width=device-width" />
      </Head>
      <ThemeProvider theme={theme}>
        <MyNavBar auth={auth} />
        {/* CssBaseline kickstart an elegant, consistent, and simple baseline to build upon. */}
        <CssBaseline />
        <Component {...pageProps} />
        <ToastContainer
          position="bottom-right"
          autoClose={5000}
          hideProgressBar={false}
          newestOnTop={false}
          closeOnClick
          rtl={false}
          pauseOnFocusLoss
          draggable
          pauseOnHover
        />
      </ThemeProvider>
    </React.Fragment>
  );
}