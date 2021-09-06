import { useRouter } from "next/router";
import React from "react";
import { toast } from "react-toastify";
import useSWR from "swr";
import FailedToConnectToBackend from "../../src/BackendConnectionFail";
import { API_URL } from "../../src/config";
import FileExplorer from "../../src/FileExplorer";
import { BasicResponse } from "../../src/Response";
const fetcher = (url) => fetch(url).then((r) => r.json());

export default function Browse() {
  const { data, error } = useSWR(API_URL + "/storages.json", fetcher);

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
    return <FileExplorer directory="/" files={myData.data} />;
  } else {
    return <div>500 server side errror</div>;
  }
}
