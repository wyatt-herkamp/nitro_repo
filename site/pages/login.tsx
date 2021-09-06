import Avatar from "@material-ui/core/Avatar";
import Box from "@material-ui/core/Box";
import Button from "@material-ui/core/Button";
import Container from "@material-ui/core/Container";
import CssBaseline from "@material-ui/core/CssBaseline";
import Grid from "@material-ui/core/Grid";
import Link from "@material-ui/core/Link";
import { makeStyles } from "@material-ui/core/styles";
import TextField from "@material-ui/core/TextField";
import Typography from "@material-ui/core/Typography";
import LockOutlinedIcon from "@material-ui/icons/LockOutlined";
import { Alert, AlertTitle } from "@material-ui/lab";
import { useRouter } from "next/dist/client/router";
import React, { useState } from "react";
import { Cookies } from "react-cookie";
import { toast } from "react-toastify";
import useSWR from "swr";
import { API_URL } from "../src/config";
import { AuthToken, BasicResponse, LoginRequest } from "../src/Response";
const useStyles = makeStyles((theme) => ({
  paper: {
    marginTop: theme.spacing(8),
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
  },
  avatar: {
    margin: theme.spacing(1),
    backgroundColor: theme.palette.secondary.main,
  },
  form: {
    width: "100%", // Fix IE 11 issue.
    marginTop: theme.spacing(3),
  },
  submit: {
    margin: theme.spacing(3, 0, 2),
  },
}));
export function LoginPage(props) {
  const classes = useStyles();
  const [login, setLoginState] = useState("login-state");

  const loginRequest = async (event) => {
    event.preventDefault(); // don't redirect the page
    // where we'll add our form logic
    let newUser = {
      username: event.target.name.value,
      password: event.target.password.value,
    };
    let body = JSON.stringify(newUser);
    console.log(body);
    const res = await fetch(API_URL + "/api/login", {
      body: body,
      headers: {
        "Content-Type": "application/json",
      },
      method: "POST",
    });
    const result = await res.json();
    let value = JSON.stringify(result);
    console.log(value);

    let response: BasicResponse<Object> = JSON.parse(value);

    if (!response.success) {
      toast.error("Invalid Username and Password", {
        position: "bottom-right",
        autoClose: 5000,
        hideProgressBar: false,
        closeOnClick: true,
        pauseOnHover: true,
        draggable: true,
        progress: undefined,
      });
    } else {
      let loginRequest = response as BasicResponse<AuthToken>;
      const cookies = new Cookies();
      let date = new Date(loginRequest.data.expiration * 1000);
      cookies.set("auth_token", loginRequest.data.token, {
        expires: date,
        sameSite: "lax",
      });
      console.log(cookies.get("auth_token"));
      setLoginState("SUCCESS");
    }
  };
  if (login == "SUCCESS") {
    const router = useRouter();
    router.push("/");
  }

  return (
    <Container component="main" maxWidth="xs">
      <CssBaseline />
      <div className={classes.paper}>
        <Avatar className={classes.avatar}>
          <LockOutlinedIcon />
        </Avatar>
        <Typography component="h1" variant="h5">
          Login
        </Typography>

        <form className={classes.form} noValidate onSubmit={loginRequest}>
          <Grid container spacing={2}>
            <Grid item xs={12}>
              <TextField
                variant="outlined"
                required
                fullWidth
                id="name"
                label="Username or Email"
                name="name"
                autoComplete="username"
              />
            </Grid>
            <Grid item xs={12}>
              <TextField
                variant="outlined"
                required
                fullWidth
                name="password"
                label="Password"
                type="password"
                id="password"
                autoComplete="current-password"
              />
            </Grid>
          </Grid>
          <Button
            type="submit"
            fullWidth
            variant="contained"
            color="primary"
            className={classes.submit}
          >
            Login
          </Button>
        </form>
      </div>
      <Box mt={5}></Box>
    </Container>
  );
}

const fetcher = (url) => fetch(url).then((r) => r.json());
export default function Install() {
  return <LoginPage />;
}
