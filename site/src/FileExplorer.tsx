import React from "react";
import { makeStyles, Theme } from "@material-ui/core/styles";
import Tabs from "@material-ui/core/Tabs";
import Tab from "@material-ui/core/Tab";
import Typography from "@material-ui/core/Typography";
import Box from "@material-ui/core/Box";
import ShowStorages from "../src/Storages";
import ShowRepos from "../src/Repositories";

export default function FileExplorer({ directory, files }) {
  let dir = directory;
  return (
    <div>
      <h1>{directory}</h1>
      {files.map((value, index) => (
        <div>
          <a href={"/browse" + dir + value}>{value}</a>
        </div>
      ))}
    </div>
  );
}
