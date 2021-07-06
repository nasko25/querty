<template>
<div>
    results
    <ul id = "results-list">
        <li class = "result" v-for="(result, index) in results" :key = "index">
            <!-- TODO &hellip; should only be added if the text was truncated -->
            <p> {{ result.title === undefined || result.title === "" ? truncate(result.text, 2) + "&hellip;" : result.title }} </p>
            <p> {{ result }} </p>
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
        },
        truncate(str, max_len) {
            return (str.length > max_len) ? str.substr(0, max_len - 1) : str;
        }
    },
    //beforeMount() {
    //    this.fetchResults();
    //}
}
</script>

<style scoped>
.result {
    list-style: none;
}
</style>
