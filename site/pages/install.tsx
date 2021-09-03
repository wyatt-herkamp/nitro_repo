
import React from 'react';
import Avatar from '@material-ui/core/Avatar';
import Button from '@material-ui/core/Button';
import CssBaseline from '@material-ui/core/CssBaseline';
import TextField from '@material-ui/core/TextField';
import FormControlLabel from '@material-ui/core/FormControlLabel';
import Checkbox from '@material-ui/core/Checkbox';
import Link from '@material-ui/core/Link';
import Grid from '@material-ui/core/Grid';
import Box from '@material-ui/core/Box';
import LockOutlinedIcon from '@material-ui/icons/LockOutlined';
import Typography from '@material-ui/core/Typography';
import { makeStyles } from '@material-ui/core/styles';
import Container from '@material-ui/core/Container';
import useSWR from 'swr'
import { API_URL } from '../src/config';
import { resolve } from 'url';
import { Alert, AlertTitle } from '@material-ui/lab';
import { BasicResponse } from '../src/Response';
import { toast } from 'react-toastify';
const useStyles = makeStyles((theme) => ({
    paper: {
        marginTop: theme.spacing(8),
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
    },
    avatar: {
        margin: theme.spacing(1),
        backgroundColor: theme.palette.secondary.main,
    },
    form: {
        width: '100%', // Fix IE 11 issue.
        marginTop: theme.spacing(3),
    },
    submit: {
        margin: theme.spacing(3, 0, 2),
    },
}));
export function InstallPage() {
    const classes = useStyles();
    const installSystem = async event => {
        event.preventDefault() // don't redirect the page
        // where we'll add our form logic
        let newUser = {

            name: event.target.name.value,
            username: event.target.username.value,
            email: event.target.email.value,
            password: event.target.password.value,
            password_two: event.target.password_two.value,

        };
        let body = JSON.stringify(newUser);
        console.log(body)
        const res = await fetch(
            API_URL + "/install",
            {
                body: body,
                headers: {
                    'Content-Type': 'application/json'
                },
                method: 'POST'
            }
        )
        const result = await res.json();
        let value = JSON.stringify(result);
        console.log(value)

        let response: BasicResponse<Object> = JSON.parse(value);

        if (!response.success) {
            console.log("Error");
            return (
                <Alert severity="error">
                    <AlertTitle>Error</AlertTitle>
                    {{ value }}
                </Alert>
            )
        } else {
            return (
                <Alert severity="success">
                    <AlertTitle>Success</AlertTitle>
                    Install Completed
                </Alert>
            )
        }
    }

    return (<Container component="main" maxWidth="xs">
        <CssBaseline />
        <div className={classes.paper}>
            <Avatar className={classes.avatar}>
                <LockOutlinedIcon />
            </Avatar>
            <Typography component="h1" variant="h5">
                Install Nitro Repo
            </Typography>
            <form className={classes.form} noValidate onSubmit={installSystem}>
                <Grid container spacing={2}>
                    <Grid item xs={12}>
                        <TextField
                            variant="outlined"
                            required
                            fullWidth
                            id="name"
                            label="Name"
                            name="name"
                            autoComplete="name"
                        />
                    </Grid>
                    <Grid item xs={12}>
                        <TextField
                            variant="outlined"
                            required
                            fullWidth
                            id="username"
                            label="Username"
                            name="username"
                            autoComplete="username"
                        />
                    </Grid>
                    <Grid item xs={12}>
                        <TextField
                            variant="outlined"
                            required
                            fullWidth
                            id="email"
                            label="Email Address"
                            name="email"
                            autoComplete="email"
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
                    <Grid item xs={12}>
                        <TextField
                            variant="outlined"
                            required
                            fullWidth
                            name="password_two"
                            label="Confirm Password"
                            type="password"
                            id="password_two"
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
                    Install
                </Button>
            </form>
        </div>
        <Box mt={5}>
        </Box>
    </Container>);
}


const fetcher = (url) => fetch(url).then((r) => r.json());
export default function Install() {
    const { data, error } = useSWR(API_URL + "/api/installed", fetcher)


    if (error) {
        toast.error('Unable to connect to Backend', {
            position: "bottom-right",
            autoClose: 5000,
            hideProgressBar: false,
            closeOnClick: true,
            pauseOnHover: true,
            draggable: true,
            progress: undefined,
            });
        return (
            <div>Error Connecting to Backend</div>
        )
    }
    if (!data) {
        return (
            <div>Loading the cool data</div>


        )
    }


    let jsonValue = JSON.stringify(data, null, 2);
    console.log(jsonValue);
    let myData: BasicResponse<Object> = JSON.parse(jsonValue);
    if (myData.success) {
        let myData = data as BasicResponse<boolean>;

        if (myData.data == false) {
            return (
                <InstallPage />


            )
        } else {
            return (
                <div>
                    Site Already Configured
                </div>
            )
        }
    } else {
        return (
            <div>
                500 server side errror
            </div>
        )
    }



}