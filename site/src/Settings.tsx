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
import { BasicResponse, SettingReport, UserPermissions } from "./Response";
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
export default function Settings() {
  const [settingValue, setSettings] = React.useState(undefined);
  const { data, error } = useSWR(API_URL + "/api/settings/report", fetcher);

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

  let response = data as BasicResponse<SettingReport>;
  let settings = response.data;
  if (settingValue == undefined) {
    setSettings(settings);
  }
  return <SettingsInside settings={settingValue} />
}
export function SettingsInside({ settings }) {
  const classes = useStyles();
  const [settingTab, setTab] = React.useState(0);

  const handleChange = (event: React.ChangeEvent<{}>, newValue: number) => {
    setTab(newValue);
  };

  return (
    <div className={classes.root}>
      <AppBar position="static">
        <Tabs
          value={settingTab}
          onChange={handleChange}
          aria-label="simple tabs example"
        >
          <Tab label="General" {...a11yProps(0)} />
          <Tab label="Email" {...a11yProps(1)} />
          <Tab label="Security" {...a11yProps(1)} />
        </Tabs>
      </AppBar>
      <TabPanel value={settingTab} index={0}>
        <GeneralSettings settings={settings.general} />
      </TabPanel>
      <TabPanel value={settingTab} index={1}>
        <EmailSettings settings={settings.email} />

      </TabPanel>
      <TabPanel value={settingTab} index={2}>
      </TabPanel>
    </div>
  );
}

export function EmailSettings({ settings }) {
  const classes = useStyles();
  const [formValue, setFormValue] = React.useState({
    "email.host": false,
    "email.username": false,
    "email.password": false,
    "email.encryption": false,
    "email.from": false,
    "email.port": false,
  });
  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setFormValue({ ...formValue, [event.target.name]: true });
  };
  const update = async (event) => {
    event.preventDefault(); // don't redirect the page
    // where we'll add our form logic
    const tempPlayer = new FormData(event.target);
    for (let [key, formV] of tempPlayer.entries()) {
      if (formValue[key] == false) {
        continue;
      }
      const cookies = new Cookies();
      let body = JSON.stringify({ "value": formV });
      const res = await fetch(
        API_URL + "/api/admin/setting/" + key + "/update",
        {
          body: body,
          headers: {
            "Content-Type": "application/json",
            Authorization: "Bearer " + cookies.get("auth_token"),
          },
          method: "POST",
        }
      );
      console.log(await res);
      const result = await res.json();

      let value = JSON.stringify(result);
      console.log(value);

      let response: BasicResponse<Object> = JSON.parse(value);
      if (!response.success) {
        toast.error("Unable to update " + key, {
          position: "bottom-right",
          autoClose: 1000,
          hideProgressBar: false,
          closeOnClick: true,
          pauseOnHover: true,
          draggable: true,
          progress: undefined,
        });
      } else {
        toast.info("Updated " + key, {
          position: "bottom-right",
          autoClose: 1000,
          hideProgressBar: false,
          closeOnClick: true,
          pauseOnHover: true,
          draggable: true,
          progress: undefined,
        });
      }
    }

  };
  console.log(settings.email_host)
  return (
    <form noValidate onSubmit={update}>
      <Grid container spacing={2}>
        <Grid item xs={12}>
          <TextField
            variant="outlined"
            required
            fullWidth
            id="email.host"
            label="Email Host"
            name="email.host"
            onChange={handleChange}
            defaultValue={settings.email_host.value}
          />
        </Grid>
        <Grid item xs={12}>
          <TextField
            variant="outlined"
            required
            fullWidth
            id="email.username"
            label="Email Username"
            name="email.username"
            onChange={handleChange}
            defaultValue={settings.email_username.value}

          />
        </Grid>
        <Grid item xs={12}>
          <TextField
            variant="outlined"
            required
            fullWidth
            id="email.password"
            label="Email Password"
            name="email.password"
            type="password"
            onChange={handleChange}

          />
        </Grid>
        <Grid item xs={12}>
          <TextField
            variant="outlined"
            required
            fullWidth
            id="email.port"
            label="Email Port"
            name="email.port"
            type="number"
            onChange={handleChange}
            defaultValue={settings.port.value}

          />
        </Grid>
        <Grid item xs={12}>
          <FormControl variant="outlined">
            <InputLabel id="demo-simple-select-outlined-label">
              Policy
            </InputLabel>
            <Select
              labelId="demo-simple-select-outlined-label"
              id="email.encryption"
              label="Email Encryption"
              name="email.encryption"
              defaultValue={settings.encryption}
              onChange={handleChange}
            >
              <MenuItem value="NONE">NONE</MenuItem>
              <MenuItem value="TLS">TLS</MenuItem>
            </Select>
          </FormControl>
        </Grid>
      </Grid>
      <Button type="submit" fullWidth variant="contained" color="primary">
        Update Settings
      </Button>
    </form>
  );
}
export function GeneralSettings({ settings }) {
  const classes = useStyles();
  const [formValue, setFormValue] = React.useState({
    "name.public": false,
  });
  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setFormValue({ ...formValue, [event.target.name]: true });
  };
  const update = async (event) => {
    event.preventDefault(); // don't redirect the page
    // where we'll add our form logic
    const tempPlayer = new FormData(event.target);
    for (let [key, formV] of tempPlayer.entries()) {
      if (formValue[key] == false) {
        continue;
      }
      const cookies = new Cookies();
      let body = JSON.stringify({ "value": formV });
      const res = await fetch(
        API_URL + "/api/admin/setting/" + key + "/update",
        {
          body: body,
          headers: {
            "Content-Type": "application/json",
            Authorization: "Bearer " + cookies.get("auth_token"),
          },
          method: "POST",
        }
      );
      console.log(await res);
      const result = await res.json();

      let value = JSON.stringify(result);
      console.log(value);

      let response: BasicResponse<Object> = JSON.parse(value);
      if (!response.success) {
        toast.error("Unable to update " + key, {
          position: "bottom-right",
          autoClose: 1000,
          hideProgressBar: false,
          closeOnClick: true,
          pauseOnHover: true,
          draggable: true,
          progress: undefined,
        });
      } else {
        toast.info("Updated " + key, {
          position: "bottom-right",
          autoClose: 1000,
          hideProgressBar: false,
          closeOnClick: true,
          pauseOnHover: true,
          draggable: true,
          progress: undefined,
        });
      }
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
            id="name"
            label="Name"
            name="name.public"
            autoComplete="name"
            onChange={handleChange}

            defaultValue={settings.name.value}
          />
        </Grid>
      </Grid>
      <Button type="submit" fullWidth variant="contained" color="primary">
        Update Settings
      </Button>
    </form>
  );
}