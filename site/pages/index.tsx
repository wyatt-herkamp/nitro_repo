import React from 'react';
import Container from '@material-ui/core/Container';
import Typography from '@material-ui/core/Typography';
import Box from '@material-ui/core/Box';
import Link from '../src/Link';
import { Grid } from '@material-ui/core';

export default function Index() {
    return (
        <Container maxWidth="sm">
            <Grid container alignItems="center"
                justify="center">
                <Typography variant="h4" component="h1" gutterBottom>
                    Nitro Repo
                </Typography>

            </Grid>
        </Container>
    );
}