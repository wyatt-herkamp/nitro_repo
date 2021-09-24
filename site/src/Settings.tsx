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
import { BasicResponse, UserPermissions } from "./Response";
import { toast } from "react-toastify";
import { Cookies } from "react-cookie";

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

export default function Settings() {
    let settings = SettingsValue;
return <SettingsInside settings={settings}/>
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
      </TabPanel>
      <TabPanel value={settingTab} index={2}>
      </TabPanel>
    </div>
  );
}

export function GeneralSettings({ settings }) {
  const classes = useStyles();
  const update = async (event) => {
    event.preventDefault(); // don't redirect the page
    // where we'll add our form logic

    console.log(body);
    const cookies = new Cookies();

    const res = await fetch(
      API_URL + "/api/admin/setting/{setting}/update",
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
            id="name"
            label="Name"
            name="name"
            autoComplete="name"
            defaultValue={user.name}
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
            defaultValue={user.email}
          />
        </Grid>
      </Grid>
      <Button type="submit" fullWidth variant="contained" color="primary">
        Update User
      </Button>
    </form>
  );
}