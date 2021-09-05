import React, { useState } from 'react';
import { makeStyles, Theme } from '@material-ui/core/styles';
import Tabs from '@material-ui/core/Tabs';
import Tab from '@material-ui/core/Tab';
import Typography from '@material-ui/core/Typography';
import Box from '@material-ui/core/Box';
import useSWR from 'swr';
import { toast } from 'react-toastify';
import { AppBar, Avatar, Button, Container, CssBaseline, FormControl, Grid, InputLabel, MenuItem, Select, TextField } from '@material-ui/core';
import axios from 'axios';
import Cookies from 'universal-cookie';
import { BasicResponse, UserList } from './Response';
import FailedToConnectToBackend from './BackendConnectionFail';
import { API_URL } from './config';
import UserSettings from './UserSettings';
import { Alert, AlertTitle } from '@material-ui/lab';
interface TabPanelProps {
    children?: React.ReactNode;
    index: any;
    value: any;
}

function TabPanel(props: TabPanelProps) {
    const { children, value, index, ...other } = props;

    return (
        <div
            role="tabpanel"
            hidden={value !== index}
            id={`vertical-tabpanel-${index}`}
            aria-labelledby={`vertical-tab-${index}`}
            {...other}
        >
            {value === index && (
                <Box p={3}>
                    <Typography>{children}</Typography>
                </Box>
            )}
        </div>
    );
}

function a11yProps(index: any) {
    return {
        id: `vertical-tab-${index}`,
        'aria-controls': `vertical-tabpanel-${index}`,
    };
}

const useStyles = makeStyles((theme: Theme) => ({
    root: {
        flexGrow: 1,
        backgroundColor: theme.palette.background.paper,
        display: 'flex',
    },
    tabs: {
        borderRight: `1px solid ${theme.palette.divider}`,
    },
}));
const fetcher = (url) => {
    const cookies = new Cookies();

    return axios
        .get(url, { headers: { Authorization: "Bearer " + cookies.get("auth_token") } }).then(res => res.data)
}

export default function ShowUsers() {
    const { data, error } = useSWR(API_URL + "/api/admin/user/list", fetcher)
    if (error) {
        console.log(error);
        toast.error('Bad Connection', {
            position: "bottom-right",
            autoClose: 5000,
            hideProgressBar: false,
            closeOnClick: true,
            pauseOnHover: true,
            draggable: true,
            progress: undefined,
        });
        return (<FailedToConnectToBackend />);

    }
    if (!data) {
        return (<FailedToConnectToBackend />);

    }
    let jsonValue = JSON.stringify(data, null, 2);

    console.log(jsonValue);
    let myData: BasicResponse<Object> = JSON.parse(jsonValue);
    if (myData.success) {
        let myData = data as BasicResponse<UserList>;
        let array = myData.data.users;
        return (<DisplayUsers values={array} />)
    }

}
export function DisplayUsers(values) {
    const classes = useStyles();
    const [user, setUser] = React.useState(0);

    const changeUser = (event: React.ChangeEvent<{}>, newValue: number) => {
        setUser(newValue);
    };

    return (
        <div className={classes.root}>
            <Tabs
                orientation="vertical"
                variant="scrollable"
                value={user}
                onChange={changeUser}
                aria-label="Vertical tabs example"
                className={classes.tabs}
            >

                {values.values.map((value, index) => (
                    <Tab label={value.username}  {...a11yProps(index)} />
                ))}
                <Tab label="Create new User" {...a11yProps(values.values.length)} />


            </Tabs>
            {values.values.map((value, index) => (
                <TabPanel value={user} index={index}>
                    <UserSettings user={value} />
                </TabPanel>

            ))}
            <TabPanel value={user} index={values.values.length}>
                <NewUser/>
            </TabPanel>




        </div>
    );
}
const userPage = makeStyles((theme) => ({
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
    formControl: {
        margin: theme.spacing(1),
        minWidth: 120,
    },
}));




export function NewUser() {

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
            </Avatar>
            <Typography component="h1" variant="h5">
              Create new User
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
                    Create
                </Button>
            </form>
        </div>
        <Box mt={5}>
        </Box>
    </Container>);
}