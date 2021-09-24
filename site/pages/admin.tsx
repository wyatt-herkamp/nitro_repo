import React from "react";
import { makeStyles, Theme } from "@material-ui/core/styles";
import Tabs from "@material-ui/core/Tabs";
import Tab from "@material-ui/core/Tab";
import Typography from "@material-ui/core/Typography";
import Box from "@material-ui/core/Box";
import ShowStorages from "../src/Storages";
import ShowRepos from "../src/Repositories";
import ShowUsers from "../src/Users";
import Me from "../src/Me";
import { Cookies } from "react-cookie";
import axios from "axios";
import { toast } from "react-toastify";
import useSWR from "swr";
import FailedToConnectToBackend from "../src/BackendConnectionFail";
import { API_URL } from "../src/config";
import { BasicResponse, User } from "../src/Response";
import Settings from "../src/Settings";

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
    "aria-controls": `vertical-tabpanel-${index}`,
  };
}

const useStyles = makeStyles((theme: Theme) => ({
  root: {
    flexGrow: 1,
    backgroundColor: theme.palette.background.paper,
    display: "flex",
  },
  tabs: {
    borderRight: `1px solid ${theme.palette.divider}`,
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
export default function VerticalTabs() {
  const { data, error } = useSWR(API_URL + "/api/me", fetcher);
  const [value, setValue] = React.useState(0);
  const classes = useStyles();

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

  let myUser = data as BasicResponse<User>;
  let user = myUser.data;

  const handleChange = (event: React.ChangeEvent<{}>, newValue: number) => {
    setValue(newValue);
  };

  return (
    <div className={classes.root}>
      <Tabs
        orientation="vertical"
        variant="scrollable"
        value={value}
        onChange={handleChange}
        aria-label="Vertical tabs example"
        className={classes.tabs}
      >
        <Tab label="Me" {...a11yProps(0)} />

        <Tab disabled={!user.permissions.admin} label="Storages" {...a11yProps(1)} />
        <Tab disabled={!user.permissions.admin} label="Repositories" {...a11yProps(2)} />
        <Tab disabled={!user.permissions.admin} label="Users" {...a11yProps(3)} />
        <Tab disabled={!user.permissions.admin} label="Settings" {...a11yProps(4)} />
      </Tabs>

      <TabPanel value={value} index={0}>
        <Me />
      </TabPanel>
      <TabPanel value={value} index={1}>
        <ShowStorages />
      </TabPanel>
      <TabPanel value={value} index={2}>
        <ShowRepos />
      </TabPanel>
      <TabPanel value={value} index={3}>
        <ShowUsers />
      </TabPanel>
      <TabPanel value={value} index={4}>
        <Settings/>
      </TabPanel>

    </div>
  )
}
