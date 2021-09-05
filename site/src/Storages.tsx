import React, { useState } from 'react';
import { makeStyles, Theme } from '@material-ui/core/styles';
import Tabs from '@material-ui/core/Tabs';
import Tab from '@material-ui/core/Tab';
import Typography from '@material-ui/core/Typography';
import Box from '@material-ui/core/Box';
import useSWR from 'swr';
import { API_URL } from './config';
import { toast } from 'react-toastify';
import FailedToConnectToBackend from './BackendConnectionFail';
import { BasicResponse, StorageList } from './Response';
import { Button, Grid, TextField } from '@material-ui/core';
import axios from 'axios';
import Cookies from 'universal-cookie';

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
        .get(url, { headers: { Authorization: "Bearer " + cookies.get("auth_token") } }).then(res=>res.data)
}

export default function ShowStorages() {
    const { data, error } = useSWR(API_URL + "/api/storages/list", fetcher)
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
        let myData = data as BasicResponse<StorageList>;
        let array = myData.data.storages;
        return (<DisplayStorages values={array} />)
    }

}
export function DisplayStorages(values) {
    const classes = useStyles();
    const [storage, setStorage] = React.useState(0);

    const changeStorage = (event: React.ChangeEvent<{}>, newValue: number) => {
        setStorage(newValue);
    };

    return (
        <div className={classes.root}>
            <Tabs
                orientation="vertical"
                variant="scrollable"
                value={storage}
                onChange={changeStorage}
                aria-label="Vertical tabs example"
                className={classes.tabs}
            >

                {values.values.map((value, index) => (
                    <Tab label={value.public_name}  {...a11yProps(index)} />
                ))}
                <Tab label="Create new Storage" {...a11yProps(values.values.length)} />


            </Tabs>
            {values.values.map((value, index) => (
                <TabPanel value={storage} index={index}>
                    {value.public_name}
                </TabPanel>

            ))}
            <TabPanel value={storage} index={values.values.length}>
                <NewStorage />
            </TabPanel>




        </div>
    );
}
export function DisplayStorage(storage) {

}
const newStorageSty = makeStyles((theme) => ({
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
export function NewStorage() {

    const newStorageRequest = async event => {
        event.preventDefault() // don't redirect the page
        // where we'll add our form logic
        let newUser = {
            name: event.target.name.value,
            public_name: event.target.public_name.value,
        };
        let body = JSON.stringify(newUser);
        console.log(body)
        const cookies = new Cookies();

        const res = await fetch(
            API_URL + "/api/admin/storages/add",
            {
                body: body,
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': "Bearer " + cookies.get("auth_token") 
                },
                method: 'POST'
            }
        )
        const result = await res.json();
        let value = JSON.stringify(result);
        console.log(value)

        let response: BasicResponse<Object> = JSON.parse(value);

        if (!response.success) {
            toast.error('Unable to create new Storage', {
                position: "bottom-right",
                autoClose: 5000,
                hideProgressBar: false,
                closeOnClick: true,
                pauseOnHover: true,
                draggable: true,
                progress: undefined,
            });

        } else {
            toast.info('Created new storage', {
                position: "bottom-right",
                autoClose: 5000,
                hideProgressBar: false,
                closeOnClick: true,
                pauseOnHover: true,
                draggable: true,
                progress: undefined,
            });
        }
    }

    const classes = newStorageSty();
    return (<form className={classes.form} noValidate onSubmit={newStorageRequest} >
        <Grid container spacing={2}>
            <Grid item xs={12}>
                <TextField
                    variant="outlined"
                    required
                    fullWidth
                    id="public_name"
                    label="Public Name"
                    name="public_name"
                />
            </Grid>
            <Grid item xs={12}>
                <TextField
                    variant="outlined"
                    required
                    fullWidth
                    id="name"
                    label="Storage Name"
                    name="name"
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
            Add New Storage
        </Button>
    </form>);
}