import React from 'react';
import Layout from '@theme/Layout';

import { Tab, Tabs, TabList, TabPanel } from 'react-tabs';
import 'react-tabs/style/react-tabs.css';


export default function Download(): JSX.Element {
    return (
        <Layout
            title={`Download`}
            description="Download page">
            <div style={{margin: "35px"}}>
                <Tabs>
                    <TabList>
                        <Tab>Windows</Tab>
                        <Tab>Linux</Tab>
                        <Tab disabled>MacOS</Tab>
                    </TabList>
                    <TabPanel>
                        <ul>
                            <li>Windows x86_64 <strong>OTC</strong> - <a href="/download/otc.exe">Download</a></li>
                            <li>Windows x86_64 <strong>OTVM</strong> - <a href="/download/otvm.exe">Download</a></li>
                        </ul>
                    </TabPanel>
                    <TabPanel>
                        <ul>
                            <li>Linux (Debian) x86_64 <strong>OTC.zip</strong> - <a href="/download/otc.zip">Download</a></li>
                            <li>Linux (Debian) x86_64 <strong>OTVM</strong> - <a href="/download/otvm">Download</a></li>
                        </ul>
                    </TabPanel>
                </Tabs>
            </div>
        </Layout>
    );
}