import axios from "axios";
import { useRouter } from "next/router";
import React from "react";
import { toast } from "react-toastify";
import useSWR from "swr";
import FailedToConnectToBackend from "../../src/BackendConnectionFail";
import { API_URL } from "../../src/config";
import FileExplorer from "../../src/FileExplorer";
import { BasicResponse } from "../../src/Response";
const fetcher = (url) => {

  return axios
    .get(url, {
      headers: { "accept": "application/json; charset=UTF-8" },
    })
    .then((res) => res.data);
};
export default function Index() {
  const router = useRouter();
  const { browse } = router.query;
  if (browse == undefined) {
    return <FailedToConnectToBackend />;
  }
  if (browse.length == 1) {
    const path = "/" + browse[0];
    const { data, error } = useSWR(
      API_URL + "/storages/" + browse[0]+".json",
      fetcher
    );

    if (error) {
      toast.error("Unable to connect to Backend", {
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
    console.log(data);
    let jsonValue = JSON.stringify(data, null, 2);
    console.log(jsonValue);
    let myData: BasicResponse<Object> = JSON.parse(jsonValue);
    if (myData.success) {
      let myData = data as BasicResponse<string[]>;
      return <FileExplorer directory={path + "/"} files={myData.data} />;
    } else {
      return <div>500 server side errror</div>;
    }
  } else {
    let path = "";
    if (Array.isArray(browse)) {
      browse.forEach((element) => {
        path = path + element + "/";
      });
    }
    console.log(path);
    const url = API_URL + "/storages/" + path;
    console.log(url);
    const { data, error } = useSWR(
      url,
      fetcher
    );

    if (error) {
      toast.error("Unable to connect to Backend", {
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
    console.log(data);
    let jsonValue = JSON.stringify(data, null, 2);
    console.log(jsonValue);
    let myData: BasicResponse<Object> = JSON.parse(jsonValue);
    if (myData.success) {
      let myData = data as BasicResponse<string[]>;
      return <FileExplorer directory={"/" + path} files={myData.data} />;
    } else {
      return <div>500 server side errror</div>;
    }
  }
}
