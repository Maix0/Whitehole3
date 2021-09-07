<template>
    <v-app>
        <v-item-group>
            <v-container>
                <v-row>
                    <v-col v-for="n in 3" :key="n" cols="12" md="4">
                        <v-item>
                            {{ guilds }}
                        </v-item>
                    </v-col>
                </v-row>
            </v-container>
        </v-item-group>
    </v-app>
</template>

<script lang="ts">
    import Vue from 'vue';
    import AsyncComputed from 'vue-async-computed'
    const API_URL = process.env.NODE_ENV === "production" ? "http://localhost:9955/api" : "http://whitehole.maix.me/api";

    Vue.use(AsyncComputed);

    export default Vue.extend({
        name: 'App',

        data: () => ({
            guilds_error: null
        }),
        asyncComputed: {
            async guilds() {
                let out: string[] = [];
                    fetch(`${API_URL}/guilds`)
                return out
            }
        },
        computed: {
        }
    });
</script>