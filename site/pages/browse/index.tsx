import { useRouter } from 'next/router';
import React from 'react';
import { toast } from 'react-toastify';
import useSWR from 'swr';
import FailedToConnectToBackend from '../../src/BackendConnectionFail';
import { API_URL } from '../../src/config';
const fetcher = (url) => fetch(url).then((r) => r.json());

export default function Index() {
    const { data, error } = useSWR(API_URL + "/storages.json", fetcher)

    if (error) {
        toast.error('Unable to connect to Backend', {
            position: "bottom-right",
            autoClose: 5000,
            hideProgressBar: false,
            closeOnClick: true,
            pauseOnHover: true,
            draggable: true,
            progress: undefined,
        });
        return (<FailedToConnectToBackend />);

    }
    if (!data) {
        return (<FailedToConnectToBackend />);
    }
    console.log(data)

    return (
        <h1>Hello</h1>
    );
}
