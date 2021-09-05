import { useRouter } from 'next/router';
import React from 'react';

export default function Index() {
    const router = useRouter();
    const { browse } = router.query;
    console.log(browse);
    return (
        <h1>Hello</h1>
    );
}
