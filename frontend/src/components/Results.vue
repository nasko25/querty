<template>
<div>
    results
    <ul id = "results-list">
        <li class = "result" v-for="(result, index) in results" :key = "index">
            {{result}}
        </li>
    </ul>
</div>
</template>

<script>

export default  {
    name: 'Results',
    data() {
        return {
            results: this.fetchResults()
        }
    },
    methods: {
        fetchResults() {
            const query = this.$route.query.q;
            if (query === undefined || query === "") {
                console.error("Empty query.")
                // TODO indicate error
                return [];
            }
            console.log("query:", query);

            // TODO if the query is present in the url, the website should be higher up
            // TODO also sort by rank
            fetch(`http://${window.location.hostname}:8000/query/${encodeURIComponent(query)}`, {
                    method: 'GET'
                })
                    .then(response => {
                        if (response.ok)
                            response.json()
                                .then(response => this.results = response)
                                .catch(err => console.error(err));
                        else
                            console.error("Suggest served responsed with an unexpected code: " + response.status)
            });
            return [];
        }
    },
    //beforeMount() {
    //    this.fetchResults();
    //}
}
</script>

<style scoped>
</style>
