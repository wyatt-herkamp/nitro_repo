import { Button, Container, Grid, Typography } from "@material-ui/core";
import React from "react";

export default function Redirect(props) {
  return (
    <Grid
      container
      direction="column"
      justifyContent="flex-end"
      alignItems="center"
    >
      <Grid item>
        <Typography variant="h4" component="h1" gutterBottom>
          Redirecting
        </Typography>
        <Button variant="contained" color="primary" href={props.url}>
          Click Here if auto redirect does not happen.
        </Button>
      </Grid>
    </Grid>
  );
}
