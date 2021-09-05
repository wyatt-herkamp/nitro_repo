
import React from 'react';
import { makeStyles, Theme } from '@material-ui/core/styles';
import AppBar from '@material-ui/core/AppBar';
import Tabs from '@material-ui/core/Tabs';
import Tab from '@material-ui/core/Tab';
import Typography from '@material-ui/core/Typography';
import Box from '@material-ui/core/Box';
import { FormControlLabel, Checkbox, FormControl, InputLabel, MenuItem, Select, Grid } from '@material-ui/core';

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
        'aria-controls': `simple-tabpanel-${index}`,
    };
}

const useStyles = makeStyles((theme: Theme) => ({
    root: {
        flexGrow: 1,
        backgroundColor: theme.palette.background.paper,
    },
}));

export default function RepoSettings({ repo }) {
    const classes = useStyles();
    const [repoTab, setTab] = React.useState(0);

    const handleChange = (event: React.ChangeEvent<{}>, newValue: number) => {
        setTab(newValue);
    };

    return (
        <div className={classes.root}>
            <AppBar position="static">
                <Tabs value={repoTab} onChange={handleChange} aria-label="simple tabs example">
                    <Tab label="General" {...a11yProps(0)} />
                    <Tab label="Security" {...a11yProps(1)} />
                </Tabs>
            </AppBar>
            <TabPanel value={repoTab} index={0}>
                <GeneralSettings repo={repo} />
            </TabPanel>
            <TabPanel value={repoTab} index={1}>
                Item Two
            </TabPanel>

        </div>
    );
}

export function GeneralSettings({ repo }) {
    const classes = useStyles();
    const [checked, setChecked] = React.useState({
        active: repo.settings.active,
        redeploy: repo.settings.re_deployment,

    });
    const updateRepo = async event => {
    }
    const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        setChecked({ ...checked, [event.target.name]: event.target.checked });
    };
    return (
        <form className={classes.form} noValidate onSubmit={updateRepo} >
            <Grid container spacing={2}>
                <Grid item xs={12}>
                    <FormControlLabel
                        control={
                            <Checkbox
                                onChange={handleChange}
                                checked={checked.redeploy}
                                name="redeploy"
                                color="primary"
                            />
                        }
                        label="Allow Redeployment"
                    />
                </Grid>
                <Grid item xs={12}>
                    <FormControlLabel
                        control={
                            <Checkbox
                                onChange={handleChange}
                                checked={checked.active}
                                name="active"
                                color="primary"

                            />
                        }
                        label="Active"
                    />
                </Grid>
                <Grid item xs={12}>
                    <FormControl variant="outlined" className={classes.formControl}>
                        <InputLabel id="demo-simple-select-outlined-label">Policy</InputLabel>
                        <Select
                            labelId="demo-simple-select-outlined-label"
                            id="policy"
                            label="Policy"
                            name="policy"
                            defaultValue={repo.settings.policy}
                        >
                            <MenuItem value="Mixed">Mixed</MenuItem>
                            <MenuItem value="Snapshot">Snapshot</MenuItem>
                            <MenuItem value="Release">Release</MenuItem>


                        </Select>
                    </FormControl>
                </Grid>

            </Grid>
        </form >
    );

}