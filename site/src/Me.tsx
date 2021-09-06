import React from "react";
import { makeStyles, Theme } from "@material-ui/core/styles";
import AppBar from "@material-ui/core/AppBar";
import Tabs from "@material-ui/core/Tabs";
import Tab from "@material-ui/core/Tab";
import Typography from "@material-ui/core/Typography";
import Box from "@material-ui/core/Box";
import {
  FormControlLabel,
  Checkbox,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  Grid,
  Button,
  TextField,
} from "@material-ui/core";
import { addMinutes } from "date-fns";
import { API_URL } from "./config";
import { BasicResponse, User, UserPermissions } from "./Response";
import { toast } from "react-toastify";
import { Cookies } from "react-cookie";
import axios from "axios";
import useSWR from "swr";
import FailedToConnectToBackend from "./BackendConnectionFail";

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
      id={`simple-tabpanel-${index}`}
      aria-labelledby={`simple-tab-${index}`}
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
    id: `simple-tab-${index}`,
    "aria-controls": `simple-tabpanel-${index}`,
  };
}

const useStyles = makeStyles((theme: Theme) => ({
  root: {
    flexGrow: 1,
    backgroundColor: theme.palette.background.paper,
  },
}));
const fetcher = (url) => {
  const cookies = new Cookies();

  return axios
    .get(url, {
      headers: { Authorization: "Bearer " + cookies.get("auth_token") },
    })
    .then((res) => res.data);
};

export default function Me() {
  const { data, error } = useSWR(API_URL + "/api/me", fetcher);
  if (error) {
    console.log(error);
    toast.error("Bad Connection", {
      position: "bottom-right",
      autoClose: 5000,
      hideProgressBar: false,
      closeOnClick: true,
      pauseOnHover: true,
      draggable: true,
      progress: undefined,
    });
    return <FailedToConnectToBackend />;
  }
  if (!data) {
    return <FailedToConnectToBackend />;
  }
  let jsonValue = JSON.stringify(data, null, 2);

  console.log(jsonValue);
  let myData: BasicResponse<Object> = JSON.parse(jsonValue);
  if (myData.success) {
    let myData = data as BasicResponse<User>;
    return <DisplayUser user={myData.data} />;
  }
}
export function DisplayUser({ user }) {
  const classes = useStyles();
  const [userTab, setTab] = React.useState(0);

  const handleChange = (event: React.ChangeEvent<{}>, newValue: number) => {
    setTab(newValue);
  };

  return (
    <div className={classes.root}>
        <h1>Hello {user.username}</h1>
      <AppBar position="static">
        <Tabs
          value={userTab}
          onChange={handleChange}
          aria-label="simple tabs example"
        >
          <Tab label="Password" {...a11yProps(0)} />
        </Tabs>
      </AppBar>
      <TabPanel value={userTab} index={0}>
        <ChangePassword user={user} />
      </TabPanel>
    </div>
  );
}

export function ChangePassword({ user }) {
  const classes = useStyles();
  const update = async (event) => {
    event.preventDefault(); // don't redirect the page
    // where we'll add our form logic
    let newUser = {
      password: event.target.password.value,
      password_two: event.target.password_two.value,
    };
    event.target.password.value = "";
    event.target.password_two.value = "";
    let body = JSON.stringify(newUser);
    console.log(body);
    const cookies = new Cookies();

    const res = await fetch(API_URL + "/api/admin/user/password", {
      body: body,
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + cookies.get("auth_token"),
      },
      method: "POST",
    });
    console.log(await res);
    const result = await res.json();

    let value = JSON.stringify(result);
    console.log(value);

    let response: BasicResponse<Object> = JSON.parse(value);
    if (!response.success) {
      toast.error("Unable to update user", {
        position: "bottom-right",
        autoClose: 5000,
        hideProgressBar: false,
        closeOnClick: true,
        pauseOnHover: true,
        draggable: true,
        progress: undefined,
      });
    } else {
      toast.info("Updated User", {
        position: "bottom-right",
        autoClose: 5000,
        hideProgressBar: false,
        closeOnClick: true,
        pauseOnHover: true,
        draggable: true,
        progress: undefined,
      });
    }
  };
  return (
    <form noValidate onSubmit={update}>
      <Grid container spacing={2}>
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
      <Button type="submit" fullWidth variant="contained" color="primary">
        Change Password
      </Button>
    </form>
  );
}
